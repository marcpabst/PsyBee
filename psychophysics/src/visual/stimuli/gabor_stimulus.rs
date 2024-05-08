// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::pattern_stimulus::PatternStimulus;
use super::patterns::Gabor;
use crate::visual::color::IntoRawRgba;
use crate::visual::geometry::Size;
use crate::visual::{
    color::RawRgba, geometry::ToVertices, stimuli::base_stimulus::BaseStimulus,
    window::WindowState, Window,
};

pub type GaborStimulus = PatternStimulus<Gabor>;

impl GaborStimulus {
    pub fn new<L, C>(
        window: &Window,
        shape: impl ToVertices + 'static,
        phase: f32,
        cycle_length: L,
        color: C,
    ) -> Self
    where
        L: Into<Size>,
        C: IntoRawRgba,
    {
        PatternStimulus::new_from_pattern(
            window,
            shape,
            Gabor::new(phase, cycle_length, color),
        )
    }

    pub fn set_phase(&mut self, phase: f32) -> () {
        self.pattern.lock().unwrap().set_phase(phase)
    }

    pub fn get_phase(&self) -> f32 {
        self.pattern.lock().unwrap().phase
    }

    pub fn set_cycle_length<L>(&mut self, cycle_length: L) -> ()
    where
        L: Into<Size>,
    {
        self.pattern.lock().unwrap().set_cycle_length(cycle_length)
    }

    pub fn get_cycle_length(&self) -> Size {
        self.pattern.lock().unwrap().cycle_length.clone()
    }

    pub fn set_color(&mut self, color: impl IntoRawRgba) -> () {
        self.pattern.lock().unwrap().set_color(color)
    }
}
