use psychophysics::{
    include_image, loop_frames, start_experiment,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{GratingsStimulus, ImageStimulus},
    visual::{stimuli::TextStimulus, Window},
};

fn show_vsync(window: Window) {
    let color1 = psychophysics::visual::color::SRGBA::new(
        255.0 / 255.0,
        128.0 / 255.0,
        128.0 / 255.0,
        1.0,
    );

    let color2 = psychophysics::visual::color::SRGBA::new(
        128.0 / 255.0,
        255.0 / 255.0,
        128.0 / 255.0,
        1.0,
    );

    // create text stimulus
    let mut text_stim = TextStimulus::new(
        &window,
        psychophysics::visual::stimuli::text::TextStimulusConfig {
            text: "VSYNC".into(),
            color: psychophysics::visual::color::RawRgba {
                r: 255.0 / 255.0,
                g: 128.0 / 255.0,
                b: 128.0 / 255.0,
                a: 1.0,
            },
            font_size: Size::Pixels(100.0),
            font_weight: glyphon::Weight::BOLD,
            ..Default::default()
        },
    );

    // set color
    text_stim.set_color(color1);

    let mut col_flag = false;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {

        // swap colors every frame
        if col_flag {
            text_stim.set_color(color1);
        } else {
            text_stim.set_color(color2);
        }

        col_flag = !col_flag;

        // add text stimulus to frame
        frame.add(&text_stim);
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_vsync);
}
