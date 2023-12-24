use palette::{
    convert::FromColorUnclamped, encoding::Srgb, white_point::D65, FromColor,
    IntoColor,
};
use psychophysics::{
    input::Key,
    log_extra, loop_frames, sleep, start_experiment,
    visual::{
        color::{self, LinearSRGBA, YxyA, SRGBA, XYZA},
        geometry::Circle,
        geometry::Rectangle,
        geometry::{Size, Transformation2D},
        pwindow::{Frame, Window},
        stimuli::gratings::GratingsStimulus,
        stimuli::image::ImageStimulus,
        text::{TextStimulus, TextStimulusConfig},
    },
    UnwrapDuration, UnwrapKeyPressAndDuration,
};

use palette::Srgba;

use futures_lite::FutureExt;

async fn gratings_experiment(window: Window) {
    // Next, we create all the visual stimuli we need for the experiment
    let start_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "Press space to start".into(),
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
    use palette::LinSrgb;

    let red_1 = LinSrgb::new(1.0, 0.0, 0.0);
    let red_2 = LinSrgb::new(1.5, 0.0, 0.0);

    // convert both colors to extended linear sRGB
    let red_1 = LinSrgb::from_color_unclamped(red_1);
    let red_2 = LinSrgb::from_color_unclamped(red_2);

    // sRGB primariey red
    let red_srgb = XYZA::new(0.64, 0.33, 0.03, 1.0);

    // same color in xyY
    let red_yxya: YxyA = red_srgb.into_color();
    println!(
        "red_yxya: ({}, {}, {})",
        red_yxya.x, red_yxya.y, red_yxya.luma
    );

    let red_dcip3_xyz = color::XYZA::new(0.68, 0.32, 0.00, 1.0);
    let green_dcip3_xyz = color::XYZA::new(0.265, 0.69, 0.045, 1.0);
    let blue_dcip3_xyz = color::XYZA::new(0.15, 0.06, 0.79, 1.0);

    // convert to Yxy
    let red_dcip3_yxy: color::YxyA = red_dcip3_xyz.into_color();
    let green_dcip3_yxy: color::YxyA = green_dcip3_xyz.into_color();
    let blue_dcip3_yxy: color::YxyA = blue_dcip3_xyz.into_color();

    // print them with 5 decimal places
    // red
    println!(
        "red_dcip3_yxy: ({:.5}, {:.5}, {:.5})",
        red_dcip3_yxy.x, red_dcip3_yxy.y, red_dcip3_yxy.luma
    );
    // green
    println!(
        "green_dcip3_yxy: ({:.5}, {:.5}, {:.5})",
        green_dcip3_yxy.x, green_dcip3_yxy.y, green_dcip3_yxy.luma
    );
    // blue
    println!(
        "blue_dcip3_yxy: ({:.5}, {:.5}, {:.5})",
        blue_dcip3_yxy.x, blue_dcip3_yxy.y, blue_dcip3_yxy.luma
    );

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

    let black = Srgba::new(0f32, 0f32, 0f32, 1f32);

    let mut rotation = 0.0;
    loop_frames!(window, keys = Key::Space, {
        // create frame with black background
        let mut frame = window.get_frame_with_bg_color(black);
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
