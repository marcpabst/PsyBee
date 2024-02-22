use photometry::ColorCalII;
use psychophysics::errors::PsychophysicsError;
use psychophysics::prelude::*;
use psychophysics::visual::color::RawRgba;
use psychophysics::visual::stimuli::patterns::Uniform;
use psychophysics::ExperimentManager;
use psychophysics::WindowManager;

fn calibrate(wn: WindowManager) -> Result<(), PsychophysicsError> {
    // create a window
    let window = wn.create_default_window();

    // wait 1s
    std::thread::sleep(std::time::Duration::from_secs(1));

    // on mac, the port is /dev/tty.usbmodem14101
    let mut color_cal = ColorCalII::new("/dev/tty.usbmodem00001").unwrap();
    println!("{:?}", color_cal);

    // create the stimulus (coloured rectangle)
    let mut stimulus = PatternStimulus::new(
        &window,
        Rectangle::FULLSCREEN,
        Uniform::new(RawRgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
    );

    // create 255 steps of the stimulus
    let mut steps = Vec::new();
    for i in 0..255 {
        let value = i as f32 / 255.0;
        steps.push(value);
    }

    let mut measurements = Vec::new();

    // show the stimulus
    for step in steps {
        // set the stimulus color
        let new_color = RawRgba {
            r: step,
            g: step,
            b: step,
            a: 1.0,
        };
        stimulus.pattern.set_color(new_color);
        let mut frame = window.get_frame();
        frame.add(&stimulus);
        window.submit_frame(frame);

        // get the measurement
        let measurement = color_cal.measure().unwrap();
        measurements.push(measurement);
        println!("{:?}", measurement);
    }

    // write the measurements to a file
    let mut writer = csv::Writer::from_path("calibration.csv").unwrap();
    for measurement in measurements {
        writer.serialize(measurement).unwrap();
    }
    writer.flush().unwrap();

    // show the stimulus
    // loop {
    //     let measurement = color_cal.measure().unwrap();
    //     println!("{:?}", measurement);
    // }

    Ok(())
}

fn main() {
    let mut em = pollster::block_on(ExperimentManager::new());

    em.run_experiment(calibrate);
}
