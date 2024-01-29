use psychophysics::camera;
use psychophysics::{
    include_image, loop_frames, start_experiment,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{GratingsStimulus, ImageStimulus},
    visual::{stimuli::TextStimulus, Window},
};

fn show_image(
    window: Window,
) -> Result<(), psychophysics::errors::PsychophysicsError> {
    // create a 640, height: 480 image
    let thatcher = image::DynamicImage::new_rgb8(640, 480);

    // create image stimulus
    let mut image_stim = ImageStimulus::new(
        &window,
        thatcher,
        Rectangle::new(
            Size::Pixels(-350.0),
            Size::Pixels(-320.0),
            Size::Pixels(700.0),
            Size::Pixels(500.0),
        ),
    );

    // when we pass the image to the thread, we need to clone it
    // so that we can still use it in the main thread - don't worry,
    // all stimuli can be cloned and will take care of the underlying
    // data synchronization for you
    let image_stim_clone = image_stim.clone();

    // spawn new thread
    let thread = std::thread::spawn(|| {
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

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {
        // add stimuli to frame
        frame.add(&image_stim);
    });

    // close window
    window.close();

    Ok(())
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
