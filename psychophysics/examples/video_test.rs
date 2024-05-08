use std::sync::{Arc, Mutex};

use byte_slice_cast::AsSliceOf;
use gst::{element_error, glib, prelude::*, Element};
use gst_app::AppSink;
use image::DynamicImage;
use psychophysics::{
    prelude::*,
    visual::stimuli::{
        patterns::{camera::Camera, Image, Uniform, Video},
        VideoStimulus,
    },
    ExperimentManager, WindowManager, WindowOptions,
};
use web_time::Instant;

// CONFIGURATION

const MONITOR_HZ: f64 = 60.0;
const MOVIES_PATH: &str = "/Users/marc/Downloads/Movies";

// EXPERIMENT
fn flicker_experiment(wm: WindowManager) -> Result<(), PsychophysicsError> {
    log::info!("Starting flicker experiment");

    // wait for 1 second
    std::thread::sleep(std::time::Duration::from_secs(2));

    let window = wm.create_default_window();

    // find all movies in the movies path (all files with *.mp4, *.mov, *.avi, *.mkv)
    let movies = std::fs::read_dir(MOVIES_PATH)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let extension = path.extension()?.to_str()?;
            if ["mp4", "mov", "avi", "mkv"].contains(&extension) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    println!("Found {} movies: {:?}", movies.len(), movies);

    let mut thumbnail_stims = vec![];

    for (i, movie) in movies.iter().enumerate() {
        let width = Size::ScreenWidth(0.1 * 16.0 / 9.0);
        let height = Size::ScreenWidth(0.1);

        // let video_pattern =
        //     Video::new(movie.to_str().unwrap(), 1280, 720, Some(20.0), true);

        let stim = VideoStimulus::create(
            &window,
            Rectangle::new(
                Size::ScreenWidth(-0.5),
                Size::ScreenHeight(0.5) - Size::ScreenWidth(0.11 * (i + 1) as f64),
                width,
                height,
            ),
            movie.to_str().unwrap(),
            1280,
            720,
            Some(20.0),
            Some(true),
        );

        thumbnail_stims.push(stim);
    }

    // create webcam stimulus
    //let webcam_pattern = Camera::new(1280, 720, true);

    // //webcam_pattern.play();
    // let webcam_stim: PatternStimulus<Camera> =
    //     PatternStimulus::new(&window, Rectangle::FULLSCREEN, webcam_pattern);

    loop {
        // get frame
        let mut frame = window.get_frame();

        frame.add_many(&thumbnail_stims);

        // frame.add(&webcam_stim);

        // submit the frame
        window.submit_frame(frame);
    }

    // Ok(())
}

fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = smol::block_on(ExperimentManager::new());

    em.run_experiment(flicker_experiment);
}
