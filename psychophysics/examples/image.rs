use psychophysics::{
    include_image,
    input::Key,
    loop_frames, start_experiment,
    utils::time,
    visual::geometry::Transformation2D,
    visual::{stimuli::video::VideoStimulus, stimuli::ImageStimulus, Window},
};

fn show_image(window: Window) {
    // include the image
    let thatcher = include_image!("wicked_witch.png");

    // create video stimulus
    let mut video_stim =
        VideoStimulus::new(&window, "/pkg/movie.mp4".to_string());

    // create image stimulus
    let mut image_stim = ImageStimulus::new(&window, thatcher);

    let mut angle = 0.0;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {
        // transform image by rotating it around its center
        video_stim.set_transformation(Transformation2D::RotationCenter(angle));
                // add video stimulus to frame
                 frame.add(&video_stim);
        // add image stimulus to frame
        frame.add(&image_stim);

     angle += 0.5;
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
