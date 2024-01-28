// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::loop_frames;
use crate::visual::geometry::Rectangle;
use crate::visual::stimuli::TextStimulus;
use crate::visual::window::Window;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PsychophysicsError {
    // file errors
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("File already exists and is not empty: {0}")]
    FileExistsAndNotEmptyError(String),
}

pub fn show_error_screen(window: &Window, error: PsychophysicsError) {
    let text_stim = TextStimulus::new(
        window,
        &format!("Error: {}", error),
        Rectangle::FULLSCREEN,
    );

    loop_frames!(frame from window, {
        // set frame color to red
        frame.set_bg_color(crate::visual::color::RED);
        frame.add(&text_stim);
    });
}
