use psychophysics::camera;
use psychophysics::{
    include_image, loop_frames, start_experiment,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{GratingsStimulus, ImageStimulus},
    visual::{stimuli::TextStimulus, Window},
};

fn show_image(window: Window) {
    // create a 640, height: 480 image
    let thatcher = image::DynamicImage::new_rgb8(640, 480);

    // create image stimulus
    let mut image_stim =
        ImageStimulus::new(&window, thatcher, Rectangle::FULLSCREEN);
    let image_stim_clone = image_stim.clone();
    // spawn new thread
    let thread = std::thread::spawn(move || {
        // list camras
        let camera_manager = camera::CameraManager::new();
        let cameras = camera_manager.cameras();
        // select first cameraDelphi method
        let camera = cameras.first().unwrap();
        // select first mode
        let mode = &camera.modes()[18];
        // open camera
        let stream = camera.open_with_callback(mode, move |frame| {
            let image: image::RgbImage = frame.into();

            // update image stimulus
            image_stim_clone
                .set_image(image::DynamicImage::ImageRgb8(image));
        });
    });

    // create text stimulus
    let text_stim = TextStimulus::new(
        &window,
        "Text!",
        Rectangle::new(
            Size::Pixels(-250.0),
            Size::Pixels(-250.0),
            Size::Pixels(500.0),
            Size::Pixels(500.0),
        ),
    );

    // set color
    text_stim.set_color(psychophysics::visual::color::RED);
    let mut angle = 0.0;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {

        // transform
        image_stim.set_transformation(Transformation2D::RotationCenter(angle));
        //video_stim.set_transformation(Transformation2D::RotationCenter(-angle));

        // add stimuli to frame
        frame.add(&image_stim);
        frame.add(&text_stim);

        angle += 0.5;
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
