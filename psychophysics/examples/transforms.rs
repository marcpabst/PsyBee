use psychophysics::{
    input::Key,
    log_extra, sleep, start_experiment,
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
        Size::ScreenWidth(-0.5) + Size::Pixels(50.0),
        Size::Pixels(-50.0),
        Size::ScreenWidth(1.0) - Size::Pixels(100.0),
        Size::Pixels(100.0),
    );

    let image_jpg =
        image::load_from_memory(include_bytes!("wundt.jpg")).unwrap();

    let gratings = ImageStimulus::new(&window, shape, image_jpg);

    // let gratings =
    //     GratingsStimulus::new(&window, shape, Size::Pixels(1.0 / 5.0), 0.0);

    // This is were the experiment starts. We first create a start screen that will be shown
    let start_screen = async {
        loop {
            let mut frame = Frame::new_with_bg_color(Color::BLACK);
            // add text stimulus to frame
            frame.add(&start_text);
            // submit frame
            window.submit_frame(frame).await;
        }
    };

    start_screen.or(window.wait_for_keypress(Key::Space)).await;

    // This is the trial loop that will be executed n_trials times
    loop {
        let fixiation_screen = async {
            loop {
                let mut frame = Frame::new_with_bg_color(Color::BLUE);
                // add fixation cross to frame
                frame.add(&fixation_cross);
                // submit frame
                window.submit_frame(frame).await;
            }
        };

        // first, show fixation cross for 500ms
        fixiation_screen.or(sleep(0.5)).await;

        let grating_screen = async {
            let mut rot = 0.0;
            let mut phase = 0.0;
            loop {
                rot += 0.5;
                phase += 0.1;
                let mut frame = Frame::new();
                // set phase
                //gratings.set_phase(phase);
                // set rotation
                gratings
                    .set_transformation(Transformation2D::RotationCenter(rot));
                // add word text to frame
                frame.add(&gratings);
                // submit frame
                window.submit_frame(frame).await;
            }
        };

        grating_screen.or(sleep(2.0)).await;
    }
}

fn main() {
    // run experiment
    start_experiment(gratings_experiment);
    // set log level to debug
    log::set_max_level(log::LevelFilter::Debug);
}
