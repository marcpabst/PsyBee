use psychophysics::{
    prelude::*, visual::stimuli::patterns::Uniform, ExperimentManager, WindowManager,
    WindowOptions,
};
use web_time::Instant;

// CONFIGURATION

const MONITOR_HZ: f64 = 60.0;

// EXPERIMENT
fn flicker_experiment(wm: WindowManager) -> Result<(), PsychophysicsError> {
    log::info!("Starting flicker experiment");
    // wait for 1 second
    std::thread::sleep(std::time::Duration::from_secs(2));

    // create empty vector of timestamps
    let mut times = vec![];

    let window = wm.create_default_window();

    let uniform_pattern = Uniform::new(color::WHITE);
    let mut stim = PatternStimulus::new_from_pattern(&window, Rectangle::FULLSCREEN, uniform_pattern);

    let mut current_color = color::RED;

    // loop
    let start = Instant::now();
    let mut lasttime = Instant::now();
    let mut i = 0;

    loop {
        // get frame
        let mut frame = window.get_frame();
        // draw the stimulus
        frame.add(&stim);

        // submit the frame
        window.submit_frame(frame);

        // get current timestamp
        let time = start.elapsed().as_secs_f64() * 1000.0;
        times.push(time);

        // change color every 10 frames
        if i % 1 == 0 {
            // send a zero byte to the serial port
            //port.write(&[9, 0]).expect("Write failed");
            current_color = if current_color == color::RED {
                color::GREEN
            } else {
                color::RED
            };
        }

        stim.pattern.set_color(current_color);
        i += 1;

        // break after 10 seconds
        if start.elapsed().as_secs_f64() > 100.0 {
            break;
        }
    }

    println!("Experiment finished");

    // calculate the time between each timestamp
    let diffs: Vec<f64> = times.windows(2).map(|w| w[1] - w[0]).collect();

    //  print diffs
    println!("Diffs: {:?}", diffs);
    Ok(())
}

fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = smol::block_on(ExperimentManager::new());

    em.run_experiment(flicker_experiment);
}
