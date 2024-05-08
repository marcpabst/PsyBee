use std::{
    i16,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use byte_slice_cast::*;
use derive_more::{Display, Error};
use gst::{element_error, glib, prelude::*, Element};
use gst_app::AppSink;
use image::DynamicImage;

fn main() {
    // create a shared buffer (DynamicImage)
    let image = Arc::new(Mutex::new(DynamicImage::new_rgb8(640, 480)));
    let image_clone = image.clone();

    // start new thread to run the GStreamer pipeline
    let t = std::thread::spawn(move || {
        gst::init().unwrap();

        let pipeline = gst::Pipeline::default();
        let src = gst::ElementFactory::make("videotestsrc")
            .build()
            .expect("Failed to create element 'videotestsrc'");
        let appsink = AppSink::builder()
            .caps(
                &gst::Caps::builder("video/x-raw")
                    .field("format", &gst_video::VideoFormat::Rgba.to_string())
                    .field("width", &640i32)
                    .field("height", &480i32)
                    .field("framerate", &gst::Fraction::new(30, 1))
                    .build(),
            )
            .build();

        // add elements to the pipeline
        pipeline.add_many(&[&src, &appsink.upcast_ref()]).unwrap();
        src.link(&appsink).unwrap();

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

                    // convert to a VECTOR of u8
                    let data = map.as_slice().as_slice_of::<u8>().unwrap().to_vec();

                    // write the buffer to the image
                    let mut image_clone: std::sync::MutexGuard<'_, DynamicImage> =
                        image_clone.lock().unwrap();
                    *image_clone = DynamicImage::ImageRgba8(
                        image::ImageBuffer::from_raw(640, 480, data).unwrap(),
                    );

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        let bus = pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        // start the pipeline
        pipeline
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");

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

        println!("Done!");
    });

    // wait for the thread to finish
    t.join().unwrap();
    println!("Finished");
}
