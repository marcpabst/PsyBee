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
pub struct Camera {
    // here, the image needs to be behinf Arc<Mutex<>> to be able to be shared between threads
    buffer: Arc<Mutex<Vec<u8>>>,
    dimensions: (u32, u32),
    pipeline: Option<gst::Pipeline>,
}

impl Camera {
    pub fn new(width: u32, height: u32, init: bool) -> Self {
        // start new thread to run the GStreamer pipeline

        gst::init().expect("Failed to initialize GStreamer");

        let buffer_vec = vec![0; (width * height * 4) as usize];
        let buffer = Arc::new(Mutex::new(buffer_vec));

        let mut out = Camera {
            buffer,
            dimensions: (width, height),
            pipeline: None,
        };

        if init {
            out.init();
        }

        out
    }

    pub fn init(&mut self) {
        // if the pipeline is already existing, return
        if self.pipeline.is_some() {
            return;
        }

        let buffer_clone = self.buffer.clone();
        let (width, height) = self.dimensions;

        // use gst_parse_launch to create the pipeline
        let pipeline = gst::parse::launch(
            "queue name=q0 ! videoconvert ! videoscale name=converter",
        )
        .unwrap();

        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        // create the camera source

        // for Mac OS, use avfvideosrc
        let src = gst::ElementFactory::make("avfvideosrc")
            .property("device-index", &1)
            .build()
            .expect("Failed to create source element");

        // add the source to the pipeline
        pipeline.add(&src).unwrap();
        // link the source to q0
        let q0 = pipeline.by_name("q0").expect("Failed to get q0 element");
        src.link(&q0).expect("Failed to link source to q0");

        // let src = pipeline
        //     .by_name("src")
        //     .expect("Failed to get source element");

        let appsink = AppSink::builder()
            .caps(
                &gst::Caps::builder("video/x-raw")
                    .field("format", &gst_video::VideoFormat::Bgra.to_string())
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
}

impl FillPattern for Camera {
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
            return vec4<f32>(textureSample(texture, texture_sampler, in.tex_coords).xyz, 0.5);
            //return textureSample(texture, texture_sampler, in.tex_coords);
        }
        "
        .to_string()
    }
}
