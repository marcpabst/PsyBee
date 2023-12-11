use psychophysics::{
    input::Key,
    input::KeyState,
    loop_frames, start_experiment,
    visual::{
        geometry::*,
        pwindow::{Frame, WindowHandle},
        text::{TextStimulus, TextStimulusConfig},
        Color,
    },
};

async fn stroop_experiment(window: WindowHandle) {
    // define colors for stroop task
    let colors = vec![Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW];
    let names = vec!["RED", "GREEN", "BLUE", "YELLOW"];
    let keys = vec![Key::R, Key::G, Key::B, Key::Y];

    let n_trials = 5;

    log::info!("Starting Stroop experiment");

    // First, create a vector of trials. Each trial is a tuple of (trial number, name, color, key)
    let mut trials = Vec::with_capacity(n_trials);
    for i in 0..n_trials {
        // draw a random color and name
        let i_color = fastrand::usize(..colors.len());
        let i_name = fastrand::usize(..names.len());
        trials.push((i, names[i_name], colors[i_color], keys[i_color]));
    }

    // Next, we create all the visual stimuli we need for the experiment
    let start_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "Press space to start".into(),
            color: Color::WHITE,
            ..Default::default()
        },
    );

    // You might wonder why there is a "mut" here. This makes the text stimulus mutable,
    // meaning that we can change its text and color later on.
    let mut word_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "".into(),
            font_size: Unit::Points(100.0),
            font_weight: glyphon::Weight::BOLD,
            ..Default::default()
        },
    );

    let too_slow_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "Too slow!".into(),
            color: Color::WHITE,
            ..Default::default()
        },
    );

    let end_text = TextStimulus::new(
        &window,
        TextStimulusConfig {
            text: "End of experiment!".into(),
            color: Color::BLACK,
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

    // This is were the experiment starts. We first create a start screen that will be shown
    loop_frames!(window, keys = Key::Space, {
        // create frame with black background
        let mut frame = Frame::new_with_bg_color(Color::BLACK);
        // add text stimulus to frame
        frame.add(&start_text);
        // submit frame
        window.submit_frame(frame).await;
    });

    // This is the trial loop that will be executed n_trials times
    for (i, name, color, correct_key) in trials {
        // this is the fixation screen that will be shown for 750ms
        loop_frames!(window, timeout = 0.75, {
            let mut frame = Frame::new();
            // add fixation cross to frame
            frame.add(&fixation_cross);
            // submit frame
            window.submit_frame(frame).await;
        });

        // set color and text
        word_text.set_color(color);
        word_text.set_text(name.to_string());

        // show word screen and wait for keypress or timeout after 2s
        let (key, duration) = loop_frames!(window, keys = keys.clone(), timeout = 2.0, {
            let mut frame = Frame::new();
            // add word text to frame
            frame.add(&word_text);
            // submit frame
            window.submit_frame(frame).await;
        });

        // check if key was pressed
        if let Some(key) = key {
            // check if key was correct
            if key == correct_key {
                log::info!("Trial {} - Correct keypress after {:?}", i + 1, duration);
            } else {
                log::info!("Trial {} - Wrong keypress after {:?}", i + 1, duration);
            }
        } else {
            log::info!("Trial {} - No keypress after {:?}", i + 1, duration);

            // show too slow screen for 500ms
            loop_frames!(window, timeout = 0.5, {
                let mut frame = Frame::new_with_bg_color(Color::BLACK);
                // add text stimulus to frame
                frame.add(&too_slow_text);
                // submit frame
                window.submit_frame(frame).await;
            });
        }
    }
    // show end screen
    loop_frames!(window, keys = Key::Space, {
        let mut frame = Frame::new_with_bg_color(Color::WHITE);
        // add text stimulus to frame
        frame.add(&end_text);
        // submit frame
        window.submit_frame(frame).await;
    });

    log::info!("End of Stroop experiment");

    // close window
    window.close().await;
}

fn main() {
    // run experiment
    start_experiment(stroop_experiment);
}
