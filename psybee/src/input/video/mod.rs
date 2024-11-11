use byte_slice_cast::AsSliceOf;
use gst::element_error;
use gst::prelude::*;
use gst_app::AppSink;
use wgpu::core::pipeline;

/// Represents a video input device.
pub trait VideoSource {
    fn get_gst_source_element(&self) -> gst::Element;
}

/// Represents a video output device.
pub trait VideoOutput {
    fn get_gst_sink_element(&self) -> gst::Element;
}

/// A video pipeline.
pub struct VideoPipeline {
    /// The video source.
    source: Box<dyn VideoSource>,
    /// The video sink.
    output: Box<dyn VideoOutput>,
    /// The underlying gstreamer pipeline.
    pipeline: gst::Pipeline,
}

impl VideoPipeline {
    /// Create a new video pipeline.
    pub fn new(source: Box<dyn VideoSource>, output: Box<dyn VideoOutput>) -> Self {
        // use gst_parse_launch to create the pipeline
        let pipeline = gst::parse::launch("queue name=q0 ! videoconvert ! videoscale name=converter").unwrap();

        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        // for Mac OS, use avfvideosrc
        // #[cfg(target_os = "macos")]
        // let src = gst::ElementFactory::make("avfvideosrc").property("device-index", &1)
        //                                                   .build()
        //                                                   .expect("Failed to create source element");

        // // for Linux, use v4l2src
        // #[cfg(target_os = "linux")]
        // let src = gst::ElementFactory::make("v4l2src").property("device", &"/dev/video0")
        //                                               .build()
        //                                               .expect("Failed to create source element");

        // // for Windows, use ksvideosrc
        // #[cfg(target_os = "windows")]
        // let src = gst::ElementFactory::make("ksvideosrc").property("device-index", &1)
        //                                                  .build()
        //                                                  .expect("Failed to create source element");

        // get the source element from the source
        let src = source.get_gst_source_element();

        // add the source to the pipeline
        pipeline.add(&src).unwrap();

        // link the source to q0
        let q0 = pipeline.by_name("q0").expect("Failed to get q0 element");
        src.link(&q0).expect("Failed to link source to q0");

        // create the sink
        let sink = output.get_gst_sink_element();

        // add the appsink to the pipeline
        pipeline.add(&sink).unwrap();

        let converter = pipeline.by_name("converter").expect("Failed to get converter element");

        converter.link(&sink).expect("Failed to link converter to appsink");

        let bus = pipeline.bus().expect("Pipeline without bus. Shouldn't happen!");

        // start the pipeline
        pipeline.set_state(gst::State::Paused).expect("Unable to set the pipeline to the `Paused` state");

        // run the pipeline in a separate thread
        let pipeline_clone = pipeline.clone();

        std::thread::spawn(move || {
            log::info!("Running...");

            // keep running until an error occurs
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Eos(..) => {
                        log::info!("End of stream");
                        break;
                    }
                    MessageView::Error(err) => {
                        pipeline_clone.set_state(gst::State::Null).unwrap();
                        log::info!("Error: {:?}", err);
                    }
                    MessageView::StateChanged(s) => {
                        log::info!("State changed from {:?}: {:?} -> {:?} ({:?})",
                                   s.src().map(|s| s.path_string()),
                                   s.old(),
                                   s.current(),
                                   s.pending());
                    }
                    _ => {
                        log::info!("Other message: {:?}", msg);
                    }
                }
            }

            pipeline_clone.set_state(gst::State::Null).unwrap();
        });

        Self { source, output, pipeline }
    }

    /// Open the video pipeline. This will start streaming video from the source, but depending on the sink,
    /// you might need to also call `start` on the pipeline (e.g. for saving to a file).
    pub fn open(&self) -> Result<(), ()> {
        self.pipeline.set_state(gst::State::Playing).unwrap();
        Ok(())
    }

    /// Close the video pipeline. This will stop streaming video from the source.
    pub fn close(&self) -> Result<(), ()> {
        self.pipeline.set_state(gst::State::Null).unwrap();
        Ok(())
    }
}

pub struct VideoFileOutput {
    filename: String,
}

impl VideoFileOutput {
    pub fn new(filename: &str) -> Self {
        Self { filename: filename.to_string() }
    }
}

impl VideoOutput for VideoFileOutput {
    fn get_gst_sink_element(&self) -> gst::Element {
        gst::ElementFactory::make("filesink").property("location", &self.filename).build().unwrap()
    }
}
