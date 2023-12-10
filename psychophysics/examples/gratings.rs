use gratings::{
    input::Key,
    log_extra, sleep, start_experiment,
    visual::{
        psychophysics::GratingsStimulus,
        pwindow::{Frame, WindowHandle},
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

    let gratings = GratingsStimulus::new(&window, 100., 0.0);

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
                let mut frame = Frame::new();
                // add fixation cross to frame
                frame.add(&fixation_cross);
                // submit frame
                window.submit_frame(frame).await;
            }
        };

        // first, show fixation cross for 500ms
        fixiation_screen.or(sleep(0.5)).await;

        let grating_screen = async {
            loop {
                let mut frame = Frame::new();
                // set phase
                gratings.params.lock().await.phase += 0.01;
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
}
