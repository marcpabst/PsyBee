use std::sync::{Arc, Mutex};

use byte_slice_cast::AsSliceOf;
use gst_app::AppSink;

use image::{GenericImageView};

use super::super::pattern_stimulus::FillPattern;
use crate::{
    prelude::PsychophysicsError,
    visual::{
        Window,
    },
};
use gst::{element_error, prelude::*};

#[derive(Clone, Debug)]
pub struct Video {
    // here, the image needs to be behinf Arc<Mutex<>> to be able to be shared between threads
    buffer: Arc<Mutex<Vec<u8>>>,
    dimensions: (u32, u32),
    pipeline: Option<gst::Pipeline>,
    file_path: String,
}

impl Video {
    pub fn new_from_path(
        file_path: &str,
        width: u32,
        height: u32,
        thumbnail: Option<f32>,
        init: Option<bool>,
    ) -> Self {
        // start new thread to run the GStreamer pipeline

        gst::init().expect("Failed to initialize GStreamer");

        let buffer_vec = vec![0; (width * height * 4) as usize];
        let buffer = Arc::new(Mutex::new(buffer_vec));

        let mut out = Video {
            buffer,
            dimensions: (width, height),
            pipeline: None,
            file_path: file_path.to_string(),
        };

        if init.unwrap_or(true) {
            out.init().expect("Failed to initialize video");
        }

        if thumbnail.is_some() {
            let img = out
                .create_thumbnail(thumbnail.unwrap())
                .expect("Failed to create thumbnail");

            *out.buffer.lock().unwrap() = img;
        }

        out
    }

    pub fn init(&mut self) -> Result<(), PsychophysicsError> {
        // if the pipeline is already existing, return
        if self.pipeline.is_some() {
            log::debug!("Pipeline already initialised. Skipping initialisation.");
            return Ok(());
        }

        let buffer_clone = self.buffer.clone();
        let file_path = self.file_path.clone();
        let (width, height) = self.dimensions;

        // use gst_parse_launch to create the pipeline
        let pipeline = gst::parse::launch(
            "filesrc name=src ! decodebin name=dmux dmux. ! queue ! videoscale ! videoconvert name=converter dmux. ! queue ! audioconvert ! audioresample ! autoaudiosink",
        )
        .unwrap();

        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        let src = pipeline
            .by_name("src")
            .expect("Failed to get source element");

        // set the file to play
        src.set_property("location", &file_path);

        let appsink = AppSink::builder()
            .caps(
                &gst::Caps::builder("video/x-raw")
                    .field("format", &gst_video::VideoFormat::Bgra.to_string())
                    .field("drop", &true)
                    .field("width", &(width as i32))
                    .field("height", &(height as i32))
                    .build(),
            )
            .build();

        // add the appsink to the pipeline
        pipeline
            .add(&appsink.upcast_ref() as &gst::Element)
            .unwrap();

        let converter = pipeline
            .by_name("converter")
            .expect("Failed to get converter element");

        converter
            .link(&appsink)
            .expect("Failed to link converter to appsink");

        // add a callback to the appsink to get the frame and write it to the image
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                // Add a handler to the "new-sample" signal.
                .new_sample(move |appsink| {
                    // get the frame
                    let sample = appsink.pull_sample().expect("Failed to get sample");

                    // print the size of the buffer
                    let buffer = sample.buffer().ok_or_else(|| {
                        element_error!(
                            appsink,
                            gst::ResourceError::Failed,
                            ("Failed to get buffer from appsink")
                        );
                        gst::FlowError::Error
                    })?;

                    let map = buffer.map_readable().map_err(|_| {
                        element_error!(
                            appsink,
                            gst::ResourceError::Failed,
                            ("Failed to map buffer readable")
                        );
                        gst::FlowError::Error
                    })?;

                    // obtain the data from the buffer
                    let data = map.as_slice_of::<u8>().unwrap().to_vec();

                    // copy into buffer
                    *buffer_clone.lock().unwrap() = data;

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        let bus = pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        // start the pipeline
        pipeline
            .set_state(gst::State::Paused)
            .expect("Unable to set the pipeline to the `Paused` state");

        // run the pipeline in a separate thread
        let pipeline_clone = pipeline.clone();

        std::thread::spawn(move || {
            println!("Running...");

            // keep running until an error occurs
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Eos(..) => {
                        println!("End of stream");
                        break;
                    }
                    MessageView::Error(err) => {
                        pipeline.set_state(gst::State::Null).unwrap();
                        println!("Error: {:?}", err);
                    }
                    MessageView::StateChanged(s) => {
                        println!(
                            "State changed from {:?}: {:?} -> {:?} ({:?})",
                            s.src().map(|s| s.path_string()),
                            s.old(),
                            s.current(),
                            s.pending()
                        );
                    }
                    _ => {
                        println!("Other message: {:?}", msg);
                    }
                }
            }

            pipeline.set_state(gst::State::Null).unwrap();
        });

        self.pipeline = Some(pipeline_clone);

        Ok(())
    }

    pub fn play(&self) -> Result<(), PsychophysicsError> {
        self.pipeline
            .as_ref()
            .expect("Pipeline not initialized")
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");
        Ok(())
    }

    pub fn pause(&self) -> Result<(), PsychophysicsError> {
        self.pipeline
            .as_ref()
            .expect("Pipeline not initialized")
            .set_state(gst::State::Paused)
            .expect("Unable to set the pipeline to the `Paused` state");
        Ok(())
    }

    fn create_thumbnail(&self, thumbnail: f32) -> Result<Vec<u8>, PsychophysicsError> {
        gst::init().expect("Failed to initialize GStreamer");

        let (width, height) = self.dimensions;

        let pipeline = gst::parse::launch(
            "filesrc name=src ! decodebin ! videoconvert ! videoscale ! appsink name=sink",
        )
        .unwrap();

        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        // extract the frame
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst_video::VideoFormat::Bgra.to_string())
            .field("width", &(width as i32))
            .field("height", &(height as i32))
            .build();

        let src = pipeline
            .by_name("src")
            .expect("Failed to get source element");

        // set the file to play
        src.set_property("location", &self.file_path);

        let appsink = pipeline
            .by_name("sink")
            .expect("Failed to get sink element")
            .dynamic_cast::<AppSink>()
            .expect("Failed to cast sink to AppSink");

        appsink.set_caps(Some(&caps));

        // set pipeline to paused
        pipeline
            .set_state(gst::State::Playing)
            .expect("Unable to set the thumbnail pipeline to the `Paused` state");

        // wait for the pipeline to reach the thumbnail position
        pipeline
            .state(gst::ClockTime::from_seconds(2))
            .0
            .expect("Failed to get state");

        // seek to the thumbnail position
        let _ = pipeline.seek_simple(
            gst::SeekFlags::FLUSH,
            gst::ClockTime::from_seconds(thumbnail as u64),
        );

        // wait for the pipeline to reach the thumbnail position
        pipeline
            .state(gst::ClockTime::from_seconds(2))
            .0
            .expect("Failed to get state");

        // get the frame
        appsink.set_property("emit-signals", &true);
        let sample = appsink.pull_sample().expect("Failed to get sample");

        // get raw data
        let buffer = sample.buffer().unwrap().map_readable().unwrap();
        let data = buffer.as_slice_of::<u8>().unwrap().to_vec();

        // set pipeline to null
        pipeline
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");

        Ok(data)
    }
}

impl FillPattern for Video {
    fn texture_extent(&self, _window: &Window) -> Option<wgpu::Extent3d> {
        let (width, height) = self.dimensions;
        Some(wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        })
    }

    fn texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        Some(self.buffer.lock().unwrap().clone())
    }

    fn updated_texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        Some(self.buffer.lock().unwrap().clone())
    }

    fn uniform_buffer_data(&mut self, _window: &Window) -> Option<Vec<u8>> {
        Some(vec![0; 32])
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        @group(0) @binding(1)
        var texture: texture_2d<f32>;

        @group(0) @binding(2)
        var texture_sampler: sampler;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(textureSample(texture, texture_sampler, in.tex_coords).xyz, 1.0);
        }
        "
        .to_string()
    }
}
