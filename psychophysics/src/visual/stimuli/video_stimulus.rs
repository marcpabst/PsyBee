// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// This module contains the video stimulus, which consists of a rectangular PatternStimulus with a video texture.
use crate::{
    prelude::PsychophysicsError,
    visual::{geometry::ToVertices, Window},
};

use super::PatternStimulus;

pub type VideoStimulus = PatternStimulus<super::patterns::Video>;

impl VideoStimulus {
    /// Create a new video stimulus.
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        video_path: &str,
        video_width: u32,
        video_height: u32,
        video_thumbnail: Option<f32>,
        video_init: Option<bool>,
    ) -> Self {
        let pattern = super::patterns::Video::new_from_path(
            video_path,
            video_width,
            video_height,
            video_thumbnail,
            video_init,
        );

        Self::new_from_pattern(window, shape, pattern)
    }

    // we forward all the important methods to the pattern
    pub fn init(&self) -> Result<(), PsychophysicsError> {
        self.pattern.lock().unwrap().init()
    }

    pub fn play(&self) -> Result<(), PsychophysicsError> {
        self.pattern.lock().unwrap().play()
    }

    pub fn pause(&self) -> Result<(), PsychophysicsError> {
        self.pattern.lock().unwrap().pause()
    }
}
