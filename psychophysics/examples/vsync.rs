use psychophysics::{prelude::*, srgb_hex};

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

    loop_frames!(frame from window, keys = Key::Escape, {

        // update the color of the flicker stimulus every update_every frames

            color_state = (color_state + 1) % color_states.len();
            flicker_stim.set_color(color_states[color_state]);


        // add grating stimulus to the current frame
         frame.add(&flicker_stim);
    });

    // close window
    window.close();

    Ok(())
}

fn main() {
    start_experiment(flicker_experiment);
}
