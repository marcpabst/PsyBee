use psychophysics::{
    input::PhysicalInput, prelude::*, visual::geometry::Circle, ExperimentManager,
    WindowManager, WindowOptions,
};

// CONFIGURATION
const KEY_FREQ_UP: Key = Key::KeyF;
const KEY_FREQ_DOWN: Key = Key::KeyA;
const KEY_START: Key = Key::Space;
const KEY_STOP: Key = Key::KeyD;
const KEY_LOG: Key = Key::KeyS;

const MONITOR_HZ: f64 = 120.0;

// EXPERIMENT
fn input_experiment(wm: WindowManager) -> Result<(), PsychophysicsError> {
    log::info!("Starting flicker experiment");

    let monitors = wm.get_available_monitors();
    let monitor = monitors
        .get(1)
        .unwrap_or(monitors.first().expect("No monitor found!"));

    let window_options: WindowOptions = WindowOptions::FullscreenHighestResolution {
        monitor: Some(monitor.clone()),
        refresh_rate: Some(MONITOR_HZ),
    };

    let window = wm.create_window(&window_options);

    // set viewing distance and size of the window in mm
    window.set_viewing_distance(30.0);
    window.set_physical_width(700.00);

    // create event logger that logs events to a BIDS compatible *.tsv file
    let mut event_logger =
        BIDSEventLogger::new("events.tsv", ["type", "freq", "key"], true)?;

    // create a event receiver
    let mut event_receiver = EventReceiver::new(&window);

    // rectangle stimus
    let mut rect_stim = ShapeStimulus::new(
        &window, // the window we want to display the stimulus inSetting color to
        Circle::new(-Size::ScreenWidth(0.5), Size::ScreenHeight(0.5), 50.0),
        color::SRGBA::new(1.0, 0.0, 0.0, 1.0), // red
    );

    loop_frames!(frame from window, {
        // check if there was a MouseMove event
        let events = event_receiver.events();
        for event in events.iter() {
           // check if CurssoMovementInput event
              if let PhysicalInput::CursorMovementInput(_) = event {
                // convert to winit event
                let winit_event = event.to_winit_window_event().unwrap();
                if let winit::event::WindowEvent::CursorMoved { position, .. } = winit_event {

                    log::info!("Cursor moved to {:?}", position);

                    let transform = Transformation2D::Translation(
                        Size::Pixels(position.x),
                        -Size::Pixels(position.y),
                    );



                // set transformation
                rect_stim.set_transformation(transform);
                }


              }
        }
        // add stimuli to frame
        frame.add(&rect_stim);
    });

    Ok(())
}

fn main() {
    let mut em = smol::block_on(ExperimentManager::new());
    em.run_experiment(input_experiment);
}
