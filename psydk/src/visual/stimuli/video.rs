use std::sync::{Arc, Mutex};

use anyhow::Error;
use byte_slice_cast::*;
use gstreamer::{element_error, element_warning, prelude::*};

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    WrappedStimulus,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::psydkWindow,
};

use psydk_proc::StimulusParams;

use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::prelude::*;
use uuid::Uuid;

#[derive(StimulusParams, Clone, Debug)]
pub struct VideoParams {
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub opacity: f64,
    pub image_x: Size,
    pub image_y: Size,
}

#[derive(Debug, Clone)]
pub enum VideoState {
    NotReady,
    Playing(f64),
    Paused(f64),
    Stopped(f64),
    Errored(String),
}

#[derive(Debug)]
pub struct VideoStimulus {
    id: uuid::Uuid,

    params: VideoParams,

    image: Option<super::WrappedImage>,
    image_fit_mode: ImageFitMode,
    // TODO: find a more efficient way to update the image
    buffer: Arc<Mutex<Option<renderer::image::RgbImage>>>,
    pipeline: gstreamer::Pipeline,

    status_rx: Arc<Mutex<std::sync::mpsc::Receiver<VideoState>>>,

    last_frame_time: f64,

    transformation: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl VideoStimulus {
    pub fn from_path(path: &str, params: VideoParams) -> Self {
        let (status_tx, status_rx) = std::sync::mpsc::channel();
        let buffer = Arc::new(Mutex::new(None));
        let pipeline = Self::create_pipeline(path, status_tx, buffer.clone()).unwrap();
        Self::start_pipeline(pipeline.clone());

        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            animations: Vec::new(),
            visible: true,
            image: None,
            image_fit_mode: ImageFitMode::Fill,
            buffer,
            pipeline,
            status_rx: Arc::new(Mutex::new(status_rx)),
            last_frame_time: -1.0,

            params,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.pipeline.current_state() == gstreamer::State::Playing
    }

    pub fn play(&self) {
        self.pipeline.set_state(gstreamer::State::Playing).unwrap();
    }

    pub fn pause(&self) {
        self.pipeline.set_state(gstreamer::State::Paused).unwrap();
    }

    pub fn seek(&self, to: f64, accurate: bool, flush: bool, block: bool) {
        // combine the flags
        let mut flags = gstreamer::SeekFlags::empty();
        if accurate {
            flags |= gstreamer::SeekFlags::ACCURATE;
        }
        if flush && self.is_playing() {
            flags |= gstreamer::SeekFlags::FLUSH;
        }

        self.pipeline
            .seek_simple(flags, gstreamer::ClockTime::from_seconds(to as u64))
            .expect("Failed to seek in video pipeline");

        // if block is true, block until the seek is done
        if block {
            self.pipeline
                .state(gstreamer::ClockTime::from_seconds(5))
                .0
                .expect("Failed to block until seek is done");
        }
    }

    fn create_pipeline(
        path: &str,
        status_tx: std::sync::mpsc::Sender<VideoState>,
        buffer: Arc<Mutex<Option<renderer::image::RgbImage>>>,
    ) -> Result<gstreamer::Pipeline, Error> {
        gstreamer::init()?;

        let pipeline = gstreamer::Pipeline::default();
        let src = gstreamer::ElementFactory::make("filesrc")
            .property("location", path)
            .build()?;

        let decodebin = gstreamer::ElementFactory::make("decodebin").build()?;

        let appsink = gstreamer_app::AppSink::builder()
            .caps(
                &gstreamer_video::VideoCapsBuilder::new()
                    .format(gstreamer_video::VideoFormat::Rgb)
                    .build(),
            )
            .build();

        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                // Add a handler to the "new-sample" signal.
                .new_sample(move |appsink| {
                    // Pull the sample in question out of the appsink's buffer.
                    let sample = appsink.pull_sample().map_err(|_| gstreamer::FlowError::Eos)?;
                    let gst_buffer = sample.buffer().ok_or_else(|| {
                        element_error!(
                            appsink,
                            gstreamer::ResourceError::Failed,
                            ("Failed to get buffer from appsink")
                        );

                        gstreamer::FlowError::Error
                    })?;

                    let caps = sample.caps().expect("caps on appsink");
                    let structure = caps.structure(0).expect("structure in caps");
                    let width = structure.get::<i32>("width").expect("width in caps");
                    let height = structure.get::<i32>("height").expect("height in caps");
                    let time = gst_buffer.pts().expect("timestamp").useconds() as f64 / 1_000_000.0;

                    // At this point, buffer is only a reference to an existing memory region somewhere.
                    // When we want to access its content, we have to map it while requesting the required
                    // mode of access (read, read/write).
                    // This type of abstraction is necessary, because the buffer in question might not be
                    // on the machine's main memory itself, but rather in the GPU's memory.
                    // So mapping the buffer makes the underlying memory region accessible to us.
                    // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
                    let map = gst_buffer.map_readable().map_err(|_| {
                        element_error!(
                            appsink,
                            gstreamer::ResourceError::Failed,
                            ("Failed to map buffer readable")
                        );

                        gstreamer::FlowError::Error
                    })?;

                    let samples = map.as_slice_of::<u8>().map_err(|_| {
                        element_error!(
                            appsink,
                            gstreamer::ResourceError::Failed,
                            ("Failed to interpret buffer as array of u8")
                        );

                        gstreamer::FlowError::Error
                    })?;

                    let new_buffer = renderer::image::RgbImage::from_raw(width as u32, height as u32, samples.to_vec())
                        .expect("Failed to create image buffer from raw data");

                    // copy the new buffer into the existing buffer
                    let mut buffer = buffer.lock().unwrap();
                    *buffer = Some(new_buffer);

                    // send the status (playing) to the channel
                    status_tx.send(VideoState::Playing(time)).unwrap();

                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline.add_many([&src, &decodebin])?;
        src.link(&decodebin)?;

        // Need to move a new reference into the closure.
        // !!ATTENTION!!:
        // It might seem appealing to use pipeline.clone() here, because that greatly
        // simplifies the code within the callback. What this actually does, however, is creating
        // a memory leak. The clone of a pipeline is a new strong reference on the pipeline.
        // Storing this strong reference of the pipeline within the callback (we are moving it in!),
        // which is in turn stored in another strong reference on the pipeline is creating a
        // reference cycle.
        // DO NOT USE pipeline.clone() TO USE THE PIPELINE WITHIN A CALLBACK
        let pipeline_weak = pipeline.downgrade();
        // Connect to decodebin's pad-added signal, that is emitted whenever
        // it found another stream from the input file and found a way to decode it to its raw format.
        // decodebin automatically adds a src-pad for this raw stream, which
        // we can use to build the follow-up pipeline.
        decodebin.connect_pad_added(move |dbin, src_pad| {
            // Here we temporarily retrieve a strong reference on the pipeline from the weak one
            // we moved into this callback.
            let Some(pipeline) = pipeline_weak.upgrade() else {
                return;
            };

            // Try to detect whether the raw stream decodebin provided us with
            // just now is either audio or video (or none of both, e.g. subtitles).
            let (is_audio, is_video) = {
                let media_type = src_pad.current_caps().and_then(|caps| {
                    caps.structure(0).map(|s| {
                        let name = s.name();
                        (name.starts_with("audio/"), name.starts_with("video/"))
                    })
                });

                match media_type {
                    None => {
                        element_warning!(
                            dbin,
                            gstreamer::CoreError::Negotiation,
                            ("Failed to get media type from pad {}", src_pad.name())
                        );

                        return;
                    }
                    Some(media_type) => media_type,
                }
            };

            let insert_sink = |is_audio, is_video| -> Result<(), Error> {
                if is_audio {
                    // decodebin found a raw audiostream, so we build the follow-up pipeline to
                    // play it on the default audio playback device (using autoaudiosink).
                    let queue = gstreamer::ElementFactory::make("queue").build()?;
                    let convert = gstreamer::ElementFactory::make("audioconvert").build()?;
                    let resample = gstreamer::ElementFactory::make("audioresample").build()?;
                    let sink = gstreamer::ElementFactory::make("autoaudiosink").build()?;

                    let elements = &[&queue, &convert, &resample, &sink];
                    pipeline.add_many(elements)?;
                    gstreamer::Element::link_many(elements)?;

                    for e in elements {
                        e.sync_state_with_parent()?;
                    }

                    let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
                    src_pad.link(&sink_pad)?;
                } else if is_video {
                    // decodebin found a raw videostream, so we build the follow-up pipeline to
                    // display it using the autovideosink.
                    let queue = gstreamer::ElementFactory::make("queue").build()?;
                    let convert = gstreamer::ElementFactory::make("videoconvert").build()?;
                    let scale = gstreamer::ElementFactory::make("videoscale").build()?;

                    let elements = &[&queue, &convert, &scale, &appsink.upcast_ref()];
                    pipeline.add_many(elements)?;
                    gstreamer::Element::link_many(elements)?;

                    for e in elements {
                        e.sync_state_with_parent()?
                    }

                    // Get the queue element's sink pad and link the decodebin's newly created
                    // src pad for the video stream to it.
                    let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
                    src_pad.link(&sink_pad)?;
                }

                Ok(())
            };

            if let Err(err) = insert_sink(is_audio, is_video) {
                // The following sends a message of type Error on the bus, containing our detailed
                // error information.
                println!("Error: {err}");
            }
        });

        Ok(pipeline)
    }

    fn start_pipeline(pipeline: gstreamer::Pipeline) {
        // pipeline.set_state(gstreamer::State::Playing).unwrap();

        let bus = pipeline.bus().expect("Pipeline without bus. Shouldn't happen!");

        std::thread::spawn(move || {
            for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
                use gstreamer::MessageView;

                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        pipeline.set_state(gstreamer::State::Null).unwrap();
                        println!(
                            "Error from element {}: {}",
                            msg.src().map(|s| s.path_string()).as_deref().unwrap_or("None"),
                            err.error().to_string()
                        );
                    }
                    _ => (),
                }
            }

            pipeline.set_state(gstreamer::State::Null).unwrap();
        });
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "VideoStimulus", extends=PyStimulus)]
pub struct PyVideoStimulus();

#[pymethods]
impl PyVideoStimulus {
    #[new]
    #[pyo3(signature = (
        path,
        x,
        y,
        width,
        height,
        opacity = 1.0
    ))]
    fn __new__(
        path: &str,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        opacity: f64,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(VideoStimulus::from_path(
                path,
                VideoParams {
                    x: x.into(),
                    y: y.into(),
                    width: width.into(),
                    height: height.into(),
                    image_x: 0.0.into(),
                    image_y: 0.0.into(),
                    opacity,
                },
            )))),
        )
    }

    fn play(mut slf: PyRef<'_, Self>) {
        downcast_stimulus!(slf, VideoStimulus).play();
    }

    fn pause(mut slf: PyRef<'_, Self>) {
        downcast_stimulus!(slf, VideoStimulus).pause();
    }

    #[pyo3(signature = (to, accurate = true, flush = true, block = true))]
    fn seek(mut slf: PyRef<'_, Self>, to: f64, accurate: bool, flush: bool, block: bool) {
        downcast_stimulus!(slf, VideoStimulus).seek(to, accurate, flush, block);
    }
}

impl_pystimulus_for_wrapper!(PyVideoStimulus, VideoStimulus);

impl Stimulus for VideoStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&mut self, scene: &mut VelloScene, window: &psydkWindow) {
        if !self.visible {
            return;
        }

        // check if the video has a new frame
        let status = self.status_rx.lock().unwrap().try_iter().last();

        let mut new_frame = false;

        match status {
            Some(VideoState::NotReady) => {
                return;
            }
            Some(VideoState::Playing(time)) => {
                if time > self.last_frame_time {
                    new_frame = true;
                    self.last_frame_time = time;
                }
            }
            Some(VideoState::Paused(time)) => {
                if time > self.last_frame_time {
                    new_frame = true;
                    self.last_frame_time = time;
                }
            }
            Some(VideoState::Stopped(time)) => {
                if time > self.last_frame_time {
                    new_frame = true;
                    self.last_frame_time = time;
                }
            }
            Some(VideoState::Errored(msg)) => {
                eprintln!("Error in video stimulus: {}", msg);
                return;
            }
            _ => {}
        }

        if new_frame {
            let buffer = self.buffer.lock().unwrap();
            let image = buffer.as_ref().unwrap();
            let image = image.clone();
            self.image = Some(super::WrappedImage::from_dynamic_image(
                renderer::image::DynamicImage::ImageRgb8(image),
            ));
        } else if self.image.is_none() {
            return;
        }

        // convert physical units to pixels
        let x = self.params.x.eval(&window.physical_properties) as f64;
        let y = self.params.y.eval(&window.physical_properties) as f64;
        let width = self.params.width.eval(&window.physical_properties) as f64;
        let height = self.params.height.eval(&window.physical_properties) as f64;

        let image_offset_x = self.params.image_x.eval(&window.physical_properties) as f64;
        let image_offset_y = self.params.image_y.eval(&window.physical_properties) as f64;

        let trans_mat = self.transformation.eval(&window.physical_properties);

        scene.draw(Geom::new_image(
            self.image.as_ref().unwrap().inner().clone(),
            x,
            y,
            width,
            height,
            trans_mat.into(),
            image_offset_x,
            image_offset_y,
            ImageFitMode::Fill,
            Extend::Pad,
        ));
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transformation.clone()
    }

    fn contains(&self, x: Size, y: Size, window: &WrappedWindow) -> bool {
        let window = window.inner();
        let ix = self.params.x.eval(&window.physical_properties);
        let iy = self.params.y.eval(&window.physical_properties);
        let width = self.params.width.eval(&window.physical_properties);
        let height = self.params.height.eval(&window.physical_properties);

        let trans_mat = self.transformation.eval(&window.physical_properties);

        let x = x.eval(&window.physical_properties);
        let y = y.eval(&window.physical_properties);

        // apply transformation by multiplying the point with the transformation matrix
        let p = nalgebra::Vector3::new(x, y, 1.0);
        let p_new = trans_mat * p;

        // check if the point is inside the rectangle
        p_new[0] >= ix && p_new[0] <= ix + width && p_new[1] >= iy && p_new[1] <= iy + height
    }

    fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParamValue) {
        self.params.set_param(name, value)
    }
}
