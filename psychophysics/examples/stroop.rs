use psychophysics::{
    input::Key,
    input::KeyState,
    log_extra, start_experiment,
    visual::{
        pwindow::{Frame, WindowHandle},
        text::{TextStimulus, TextStimulusConfig},
        Color,
    },
};

macro_rules! loop_frames {
    ($win:expr $(, keys = $keys:expr)? $(, keystate = $keystate:expr)? $(, timeout = $timeout:expr)?, $body:block) => {
        {
            let timeout_duration = $(Some(web_time::Duration::from_secs_f64($timeout));)? None as Option<web_time::Duration>;
            let keys_vec: Vec<Key> = $($keys.into_iter().map(|k| k.into()).collect();)? vec![] as Vec<Key>;
            let keystate: KeyState = $($keystate;)? KeyState::Any;

            let mut keyboard_receiver = $win.keyboard_receiver.activate_cloned();

            let mut kc: Option<Key> = None;

            {
                $body
            }

            let start = web_time::Instant::now();

            'outer: loop {

                // check if timeout has been reached
                if timeout_duration.is_some() && start.elapsed() > timeout_duration.unwrap() {
                    break 'outer;
                }
                // check if a key has been pressed
                while let Ok(e) = keyboard_receiver.try_recv() {
                    // check if the key is one of the keys we are looking for
                    if keys_vec.contains(&e.virtual_keycode.unwrap().into()) && keystate == e.state.into() {
                        kc = Some(e.virtual_keycode.unwrap().clone().into());
                        break 'outer;
                    }
                }
                // if not, run another iteration of the loop
                $body
            }
        (kc, start.elapsed())
        }
    };
}

async fn stroop_experiment(window: WindowHandle) {
    // define colors for stroop task
    let colors = vec![Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW];
    let names = vec!["RED", "GREEN", "BLUE", "YELLOW"];
    let n_trials = 10;

    // First, create a vector of trials. Each trial is a tuple of (trial number, color, name)
    let mut trials = Vec::with_capacity(n_trials);
    for i in 0..n_trials {
        // draw a random color and name
        let i_color = fastrand::usize(..colors.len());
        let i_name = fastrand::usize(..names.len());
        trials.push((i, names[i_name], colors[i_color]));
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
            font_size: 100.0,
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
    loop_frames!(window, keys = [Key::Space], {
        // create frame with black background
        let mut frame = Frame::new_with_bg_color(Color::BLACK);
        // add text stimulus to frame
        frame.add(&start_text);
        // submit frame
        window.submit_frame(frame).await;
    });

    // This is the trial loop that will be executed n_trials times
    for (i, name, color) in trials {
        // this is the fixation screen that will be shown for 750ms
        loop_frames!(window, timeout = 0.75, {
            let mut frame = Frame::new();
            // add fixation cross to frame
            frame.add(&fixation_cross);
            // submit frame
            window.submit_frame(frame).await;
        });

        // set color and text
        word_text.set_color(color.into());
        word_text.set_text(name.to_string());

        let (key, duration) = loop_frames!(
            window,
            keys = [Key::R, Key::G, Key::B, Key::Y],
            timeout = 2.0,
            {
                let mut frame = Frame::new();
                // add word text to frame
                frame.add(&word_text);
                // submit frame
                window.submit_frame(frame).await;
            }
        );

        // Check if the keypress was correct
        if key.is_some() {
            log_extra!(
                "Trial {} - Key {:?} after {:?}",
                i + 1,
                key.unwrap(),
                duration
            );
        } else {
            log_extra!("Trial {} - No keypress after {:?}", i + 1, duration);

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
    loop_frames!(window, {
        let mut frame = Frame::new_with_bg_color(Color::WHITE);
        // add text stimulus to frame
        frame.add(&end_text);
        // submit frame
        window.submit_frame(frame).await;
    });
}

fn main() {
    // run experiment
    start_experiment(stroop_experiment);
}
