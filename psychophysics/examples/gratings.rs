use psychophysics::prelude::*;

use std::f32::consts::PI;

fn flicker_experiment(
    window: Window,
) -> Result<(), PsychophysicsError> {
    // wait 1s
    sleep_secs(0.5);

    // set viewing distance and size of the window in mm
    window.set_viewing_distance(30.0);
    window.set_physical_width(700.00);

    // create event logger that logs events to a BIDS compatible *.tsv file
    let mut event_logger = BIDSEventLogger::new(
        "events.tsv",
        vec!["trial_type", "stimulus"],
        true,
    )?;

    // create a key press receiver that will be used to check if the up or down key was pressed
    let mut kpr: KeyPressReceiver = KeyPressReceiver::new(&window);

    // calculate the supported flicker frequencies based on the monitor refresh rate
    let monitor_hz: f64 = 120.0;
    let (max_hz, min_hz) = (monitor_hz, 1.0);

    // find all divisors of the monitor refresh rate that are between min_hz and max_hz
    let mut divisors = Vec::new();
    for i in 1..=monitor_hz as usize {
        if monitor_hz % i as f64 == 0.0
            && i as f64 <= max_hz
            && i as f64 >= min_hz
        {
            divisors.push(i);
        }
    }

    log::info!("Supported flicker frequencies: {:?}", divisors);

    let mut current_hz_index = 5;
    let mut current_hz = divisors[current_hz_index] as f64;
    let mut update_every = (monitor_hz / current_hz) as usize;

    // create text stimulus
    let start_stim = TextStimulus::new(
        &window, // the window we want to display the stimulus in
        "Press space to start", // the text we want to display
        Rectangle::FULLSCREEN, // full screen
    );
    let freq_stim = TextStimulus::new(
        &window, // the window we want to display the stimulus in
        format!("{} Hz", current_hz), // the text we want to display
        Rectangle::FULLSCREEN, // full screen
    );

    // create grating stimulus
    let grating_stim = GratingsStimulus::new(
        &window, // the window we want to display the stimulus inSetting color to
        Rectangle::FULLSCREEN, // full screen
        Size::Pixels(500000.0), // size of one period
        0.0,     // phase in radians
    );

    // variable to store the current phase of the grating
    let mut phase = PI / 2.0;

    event_logger.log_cols(
        ("trial_type",),
        ("experiment start",),
        0.0,
    )?;

    loop {
        // show text until space key is pressed to start the experiment
        loop_frames!(frame from window, keys = Key::Space, {
            frame.add(&start_stim);
        });

        // show frames until space key is pressed
        loop_frames!((i, frame) from window, keys = Key::Space, {

            // update the phase
            if i % update_every == 0 {
                phase += PI;
                grating_stim.set_phase(phase);
            }

            // add grating stimulus to the current frame
            frame.add(&grating_stim);
            frame.add(&freq_stim);

            // log event
            event_logger.log(("flicker", current_hz), 1.0 / current_hz)?;

            // check if the up or down key was pressed
            let key = kpr.get_keys();
            if key.iter().any(|k| k.key == Key::Up && k.state == KeyState::Pressed) {
                if current_hz_index < divisors.len() - 1 {
                    current_hz_index += 1;
                    current_hz = divisors[current_hz_index] as f64;
                    update_every = (monitor_hz / current_hz) as usize;
                    freq_stim.set_text(format!("{} Hz", current_hz));
                }
            } else if key.iter().any(|k| k.key == Key::Down && k.state == KeyState::Pressed) {
                if current_hz_index > 0 {
                    current_hz_index -= 1;
                    current_hz = divisors[current_hz_index] as f64;
                    update_every = (monitor_hz / current_hz) as usize;
                    freq_stim.set_text(format!("{} Hz", current_hz));
                }
            }
        });
    }
}

fn main() {
    // run experiment
    start_experiment(flicker_experiment);
}
