use palette::{convert::FromColorUnclamped, white_point::D65, FromColor};
use psychophysics::{
    input::Key,
    log_extra, loop_frames, sleep, start_experiment,
    visual::{
        geometry::Circle,
        geometry::Rectangle,
        geometry::{Size, Transformation2D},
        pwindow::{Frame, WindowHandle},
        stimuli::gratings::GratingsStimulus,
        stimuli::image::ImageStimulus,
        text::{TextStimulus, TextStimulusConfig},
        Color,
    },
    UnwrapDuration, UnwrapKeyPressAndDuration,
};

use futures_lite::FutureExt;

async fn gratings_experiment(window: WindowHandle) {
    // Next, we create all the visual stimuli we need for the experiment
    let start_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "Press space to start".into(),
            color: Color::WHITE,
            ..Default::default()
        },
    );

    let fixation_cross = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "+".to_string(),
            ..Default::default()
        },
    );

    let shape = Rectangle::new(
        Size::Pixels(-250.0),
        Size::Pixels(-250.0),
        Size::Pixels(500.0),
        Size::Pixels(500.0),
    );

    let image_jpg =
        image::load_from_memory(include_bytes!("wicked_witch.png")).unwrap();

    let gratings = ImageStimulus::new(&window, shape, image_jpg);

    // create an array of RGBA values in f16 format
    use half::f16;
    use palette::{Lab, LinSrgb, Srgb};

    let red_1 = LinSrgb::new(1.0, 0.0, 0.0);
    let red_2 = LinSrgb::new(1.5, 0.0, 0.0);

    // convert both colors to extended linear sRGB
    let red_1 = LinSrgb::from_color_unclamped(red_1);
    let red_2 = LinSrgb::from_color_unclamped(red_2);

    // create a vector of 500 x 500 rgba values, with the first half being red_1
    // and the second half being red_2
    let mut rgba: Vec<f16> = Vec::new();
    for i in 0..500 {
        for j in 0..500 {
            if i < 250 {
                rgba.push(f16::from_f32(red_1.red));
                rgba.push(f16::from_f32(red_1.green));
                rgba.push(f16::from_f32(red_1.blue));
                rgba.push(f16::from_f32(1.0));
            } else {
                rgba.push(f16::from_f32(red_2.red));
                rgba.push(f16::from_f32(red_2.green));
                rgba.push(f16::from_f32(red_2.blue));
                rgba.push(f16::from_f32(1.0));
            }
        }
    }
    // create a slice from the vector
    let rgba = rgba.as_slice();

    gratings.set_texture(rgba);

    // let gratings =
    //     GratingsStimulus::new(&window, shape, Size::Pixels(1.0 / 5.0), 0.0);

    let mut rotation = 0.0;
    loop_frames!(window, keys = Key::Space, {
        // create frame with black background
        let mut frame = Frame::new_with_bg_color(Color::BLACK);
        // add the image stimulus to the frame
        frame.add(&gratings);
        // rotate the image
        //rotation += 0.1;
        gratings.set_transformation(Transformation2D::RotationCenter(rotation));
        // submit frame
        window.submit_frame(frame).await;
    });
}

fn main() {
    // run experiment
    start_experiment(gratings_experiment);
    // set log level to debug
    log::set_max_level(log::LevelFilter::Debug);
}
