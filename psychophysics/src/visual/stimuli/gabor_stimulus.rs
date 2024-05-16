// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::pattern_stimulus::PatternStimulus;
use super::patterns::Gabor;
use crate::generate_assessors;
use crate::visual::color::IntoRawRgba;
use crate::visual::geometry::Size;
use crate::visual::{
    color::RawRgba, geometry::ToVertices, Window,
};

pub type GaborStimulus = PatternStimulus<Gabor>;

impl GaborStimulus {
    pub fn new<L, C, M, N>(
        window: &Window,
        shape: impl ToVertices + 'static,
        phase: f32,
        cycle_length: L,
        std_x: M,
        std_y: N,
        orientation: f32,
        color: C,
    ) -> Self
    where
        L: Into<Size>,
        M: Into<Size>,
        N: Into<Size>,
        C: IntoRawRgba,
    {
        PatternStimulus::new_from_pattern(
            window,
            shape,
            Gabor::new(phase, cycle_length, std_x, std_y, orientation, color),
        )
    }

    generate_assessors!(pattern, phase, f32);
    generate_assessors!(pattern, cycle_length, Into<Size>);
    generate_assessors!(pattern, std_x, Into<Size>);
    generate_assessors!(pattern, std_y, Into<Size>);
    generate_assessors!(pattern, orientation, f32);

    pub fn set_color(&mut self, color: impl IntoRawRgba) -> () {
        self.pattern.lock().unwrap().set_color(color)
    }

    pub fn color(&self) -> RawRgba {
        self.pattern.lock().unwrap().color
    }
}
