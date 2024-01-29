// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use psychophysics::{
    include_image, loop_frames, onnx, start_experiment,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{GratingsStimulus, ImageStimulus},
    visual::{stimuli::TextStimulus, Window},
};

fn show_image(window: Window) {
    // include the image
    let thatcher = include_image!("wicked_witch.png");

    // create image stimulus
    let mut image_stim =
        ImageStimulus::new(&window, thatcher, Rectangle::FULLSCREEN);

    // create text stimulus
    let text_stim = TextStimulus::new(
        &window,
        "Ding Dong!",
        Rectangle::new(
            Size::Pixels(-250.0),
            Size::Pixels(-250.0),
            Size::Pixels(500.0),
            Size::Pixels(500.0),
        ),
    );

    // set color
    text_stim.set_color(psychophysics::visual::color::RED);
    let mut angle = 0.0;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {

        // transform
        image_stim.set_transformation(Transformation2D::RotationCenter(angle));
        //video_stim.set_transformation(Transformation2D::RotationCenter(-angle));

        // add stimuli to frame
        frame.add(&image_stim);
        frame.add(&text_stim);

        angle += 0.5;
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
