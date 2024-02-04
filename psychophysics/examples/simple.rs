use psychophysics::{prelude::*, ExperimentManager, WindowManager, WindowOptions};

const MONITOR_HZ: f64 = 60.0;

// EXPERIMENT
fn flicker_experiment(wm: WindowManager) -> Result<(), PsychophysicsError> {
    let monitors = wm.get_available_monitors();
    let monitor = monitors
        .get(1)
        .unwrap_or(monitors.first().expect("No monitor found!"));

    let window_options: WindowOptions = WindowOptions::FullscreenHighestResolution {
        monitor: Some(monitor.clone()),
        refresh_rate: Some(MONITOR_HZ),
    };

    let window = wm.create_window(&window_options);

    let stim = GratingsStimulus::new(
        &window,
        Rectangle::FULLSCREEN,
        GratingType::Sine {
            phase: 0.0,
            cycle_length: Size::Pixels(20.0),
        },
    );

    loop {
        let mut frame = window.get_frame();
        frame.add(&stim);
        window.submit_frame(frame);
    }

    Ok(())
}

fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = smol::block_on(ExperimentManager::new());

    em.run_experiment(flicker_experiment);
}
