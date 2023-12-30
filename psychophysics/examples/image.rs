use psychophysics::{
    include_image,
    input::Key,
    loop_frames, start_experiment,
    utils::time,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{GratingsStimulus, ImageStimulus, VideoStimulus},
    visual::Window,
};

fn show_image(window: Window) {
    // include the image
    let thatcher = include_image!("wicked_witch.png");

    // create image stimulus
    let mut image_stim = ImageStimulus::new(&window, thatcher);
    // create video stimulus
    // let mut video_stim = VideoStimulus::new_with_rectangle(
    //     &window,
    //     "movie.mp4".to_string(),
    //     Rectangle::new(
    //         Size::Pixels(250.0),
    //         Size::Pixels(250.0),
    //         Size::Pixels(500.0),
    //         Size::Pixels(500.0),
    //     ),
    // );

    // create grating stimulus
    let mut grating_stim = GratingsStimulus::new(
        &window,
        Rectangle::new(
            Size::Pixels(-250.0),
            Size::Pixels(-250.0),
            Size::Pixels(500.0),
            Size::Pixels(500.0),
        ),
        Size::Pixels(100.0),
        0.0,
    );

    let mut angle = 0.0;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {

        // transform
        image_stim.set_transformation(Transformation2D::RotationCenter(angle));
        //video_stim.set_transformation(Transformation2D::RotationCenter(-angle));

        // add stimuli to frame
        //frame.add(&grating_stim);
        frame.add(&image_stim);
        //frame.add(&video_stim);

        angle += 0.5;
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
