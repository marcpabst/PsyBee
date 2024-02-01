use mobile_entry_point::mobile_entry_point;
use psychophysics::{
    prelude::*, srgb_hex, ExperimentManager, WindowOptions,
};

const MONITOR_HZ: f64 = 60.0;

// EXPERIMENT
fn flicker_experiment(
    window: Window,
) -> Result<(), PsychophysicsError> {
    // create flicker stim
    let red = srgb_hex!(0xDE8A85);
    let cyan = srgb_hex!(0xB6FAFC);
    let color_states = vec![red, cyan];
    let mut color_state = 0;
    let flicker_stim = ShapeStimulus::new(
        &window, // the window we want to display the stimulus inSetting color to
        Rectangle::FULLSCREEN, // full screen
        color_states[color_state], // the color of the stimulus
    );

    // create a key press receiver that will be used to check if the up or down key was pressed
    let mut kpr = KeyPressReceiver::new(&window);

    loop_frames!(frame from window, keys = Key::Escape, {

        // update the color of the flicker stimulus every update_every frames

        color_state = (color_state + 1) % color_states.len();
        flicker_stim.set_color(color_states[color_state]);


        // check if the space key was pressed
        if kpr.get_keys().was_pressed(Key::Space) {
            // if so, skip one frame
            color_state = (color_state + 1) % color_states.len();
        }
        // add grating stimulus to the current frame
         frame.add(&flicker_stim);
    });

    // close window
    window.close();

    Ok(())
}

#[mobile_entry_point]
fn main() {
    // create experiment manager
    let mut em = ExperimentManager::new();

    // get all available monitors
    let monitors = em.get_available_monitors();

    // select the second monitor if available, otherwise use the primary one
    let monitor = monitors
        .get(1)
        .unwrap_or(monitors.first().expect("No monitor found!"));

    // create window options (here, we use the highest resolution of the chosen monitor)
    let window_options = WindowOptions::FullscreenHighestResolution {
        monitor: Some(monitor.clone()),
        refresh_rate: Some(MONITOR_HZ),
    };
    // start experiment (this will block until the experiment is finished)
    em.run_experiment(&window_options, flicker_experiment);
}
