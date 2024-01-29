use psychophysics::prelude::*;

// CONFIGURATION
const KEY_FREQ_UP: Key = Key::F;
const KEY_FREQ_DOWN: Key = Key::A;
const KEY_START: Key = Key::Space;
const KEY_STOP: Key = Key::D;
const KEY_LOG: Key = Key::S;

const MONITOR_HZ: f64 = 60.0; // TODO: get this from the window

fn find_possible_refresh_rates(
    refresh_rate: f64,
    min_rate: f64,
) -> Vec<f64> {
    let mut possible_rates = Vec::new();

    // step throug all integers up to the refresh rate
    for i in 1..(refresh_rate as usize) {
        let rate = refresh_rate / i as f64;
        if rate >= min_rate * 2.0 {
            possible_rates.push(rate / 2.0);
        }
    }

    possible_rates
}
// EXPERIMENT
fn flicker_experiment(
    window: Window,
) -> Result<(), PsychophysicsError> {
    // set viewing distance and size of the window in mm
    window.set_viewing_distance(30.0);
    window.set_physical_width(700.00);

    // wait 1s (TODO: remove this)
    sleep_secs(1.0);

    // create event logger that logs events to a BIDS compatible *.tsv file
    let mut event_logger = BIDSEventLogger::new(
        "events.tsv",
        ["type", "freq", "key"],
        true,
    )?;

    // open serial port
    let mut serial_port = SerialPort::open_or_dummy(
        "/dev/ttyACM0".to_string(),
        9600,
        1000,
    );

    event_logger.log_cols(("type",), ("experiment start",), 0.0)?;

    // create a key press receiver that will be used to check if the up or down key was pressed
    let mut kpr: KeyPressReceiver = KeyPressReceiver::new(&window);

    // find all available freqs for the monitor by dividing the monitor hz by 2 until we reach 1
    let mut available_freqs =
        find_possible_refresh_rates(MONITOR_HZ, 1.0);

    log::info!("Available freqs: {:?}", available_freqs);

    // setup color and freqs
    let mut current_hz_index = 0;
    let mut current_hz = available_freqs[current_hz_index] as f64;
    let mut update_every = (MONITOR_HZ / current_hz / 2.0) as usize;

    let color_states = vec![color::BLACK, color::WHITE];
    let mut color_state: usize = 0;

    // create text stimulus
    let start_stim = TextStimulus::new(
        &window, // the window we want to display the stimulus in
        "Press space to start", // the text we want to display
        Rectangle::FULLSCREEN, // full screen
    );

    let freq_stim = TextStimulus::new(
        &window, // the window we want to display the stimulus in
        format!("{} Hz", current_hz), // the text we want to display
        Rectangle::new(
            -Size::ScreenWidth(0.5),
            -Size::ScreenHeight(0.5),
            Size::Pixels(500.0),
            Size::Pixels(500.0),
        ), // full screen
    );

    // create flicker stim
    let flicker_stim = ShapeStimulus::new(
        &window, // the window we want to display the stimulus inSetting color to
        Rectangle::FULLSCREEN, // full screen
        color_states[color_state], // the color of the stimulus
    );

    loop {
        // show text until space key is pressed to start the experiment
        loop_frames!(frame from window, keys = KEY_START, {
            frame.add(&start_stim);
        });

        event_logger.log_cols(("type",), ("block start",), 0.0)?;

        // show frames until space key is pressed
        loop_frames!((i, frame) from window, keys = KEY_STOP, {

            // update the color of the flicker stimulus every update_every frames
            if i % update_every == 0 {
                color_state = (color_state + 1) % color_states.len();
                flicker_stim.set_color(color_states[color_state]);
            }

            // add grating stimulus to the current frame
             frame.add(&flicker_stim);
            frame.add(&freq_stim);

            // log event
            event_logger.log_cols(
                 ("type", "freq"),
                ("flicker", current_hz),
                0.0,
            )?;

            // get all keys that were pressed since the last frame
            let keys = kpr.get_keys();

            if !keys.is_empty() {
                if keys.was_pressed(KEY_LOG) {
                    event_logger.log_cols(("type", "key"), ("keydown", "space"), 0.0)?;
                } else if keys.was_released(KEY_LOG) {
                    event_logger.log_cols(("type", "key"), ("keyup", "space"), 0.0)?;
                }

                if keys.was_pressed(KEY_FREQ_UP) {
                    current_hz_index = (current_hz_index + 1) % available_freqs.len();

                } else if keys.was_pressed(KEY_FREQ_DOWN) {
                    current_hz_index = (current_hz_index + available_freqs.len() - 1) % available_freqs.len();

                }
                current_hz = available_freqs[current_hz_index] as f64;
                update_every = (MONITOR_HZ / current_hz / 2.0) as usize;
                freq_stim.set_text(format!("{:.2} Hz", current_hz));
            }



        });
    }
}

fn main() {
    start_experiment(flicker_experiment);
}
