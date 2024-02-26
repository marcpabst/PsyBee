use psychophysics::{
    prelude::*, visual::stimuli::patterns::Uniform, ExperimentManager, WindowManager,
    WindowOptions,
};

// CONFIGURATION

const MONITOR_HZ: f64 = 60.0;

// EXPERIMENT
fn flicker_experiment(wm: WindowManager) -> Result<(), PsychophysicsError> {
    log::info!("Starting flicker experiment");
    // wait for 1 second
    std::thread::sleep(std::time::Duration::from_secs(2));

    // setup serial port

    let port_name = "/dev/cu.usbmodem11301";
    let mut port = serialport::new(port_name, 256000)
        .timeout(std::time::Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let monitors = wm.get_available_monitors();
    let monitor = monitors
        .get(1)
        .unwrap_or(monitors.first().expect("No monitor found!"));

    let window_options: WindowOptions = WindowOptions::FullscreenHighestResolution {
        monitor: Some(monitor.clone()),
        refresh_rate: Some(MONITOR_HZ),
    };

    let window = wm.create_default_window();

    let uniform_pattern = Uniform::new(color::WHITE);
    let mut stim = PatternStimulus::new(&window, Rectangle::FULLSCREEN, uniform_pattern);

    // loop
    let mut i = 0;
    loop {
        // get frame
        let mut frame = window.get_frame();
        // draw the stimulus
        frame.add(&stim);

        // submit the frame
        window.submit_frame(frame);

        // change color every 10 frames
        if i % 10 == 0 {
            // send a zero byte to the serial port
            port.write(&[0]).expect("Write failed");
            stim.pattern.set_color(color::WHITE);
        } else {
            stim.pattern.set_color(color::BLACK);
        }

        i += 1;
    }
}

fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = smol::block_on(ExperimentManager::new());

    em.run_experiment(flicker_experiment);
}
