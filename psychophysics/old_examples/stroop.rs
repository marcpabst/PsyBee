use psychophysics::{
    input::Key,
    loop_frames, start_experiment,
    visual::{color, geometry::*, stimuli::TextStimulus, window::Window},
};
fn stroop_experiment(window: Window) {
    let colors = vec![color::RED, color::GREEN, color::BLUE, color::YELLOW];
    let names = vec!["RED", "GREEN", "BLUE", "YELLOW"];
    let keys = vec![Key::R, Key::G, Key::B, Key::Y];

    // create a vector of trials
    let n_trials = 10;
    let mut trials = Vec::new();

    for i in 0..n_trials {
        // draw a random color and name
        let i_color = fastrand::usize(..colors.len());
        let i_name = fastrand::usize(..names.len());
        trials.push((i, names[i_name], colors[i_color], keys[i_color]));
    }

    // Next, we create all the visual stimuli we need for the experiment
    let start_text =
        TextStimulus::new(&window, "Press space to start", Rectangle::FULLSCREEN);
    let mut word_text = TextStimulus::new(&window, "placeholder", Rectangle::FULLSCREEN);
    let too_slow_text = TextStimulus::new(&window, "Too slow!", Rectangle::FULLSCREEN);
    let end_text = TextStimulus::new(&window, "End of experiment", Rectangle::FULLSCREEN);
    let fixation_cross = TextStimulus::new(&window, "+", Rectangle::FULLSCREEN);

    loop_frames!(frame from window, keys = Key::Space, {
        // add text stimulus to frame
        frame.add(&start_text);
    });

    // This is the trial loop that will be executed n_trials times
    for (i, name, color, correct_key) in trials {
        // this is the fixation screen that will be shown for 750ms
        loop_frames!(frame from window, timeout = 0.75, {
            // add fixation cross to frame
            frame.add(&fixation_cross);
        });

        // set color and text
        word_text.set_color(color);
        word_text.set_text(name.to_string());

        // show word screen and wait for keypress or timeout after 2s
        let (key, duration) = loop_frames!(frame from window, keys = keys.clone(), timeout = 2.0, {
            // add word text to frame
            frame.add(&word_text);
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
            loop_frames!(frame from window, timeout = 0.5, {
                // add text stimulus to frame
                frame.add(&too_slow_text);
            });
        }
    }

    // show end screen
    loop_frames!(frame from window, keys = Key::Space, {
        // add text stimulus to frame
        frame.add(&end_text);
    });

    log::info!("End of Stroop experiment");

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(stroop_experiment);
}
