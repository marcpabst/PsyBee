// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::pattern_stimulus::PatternStimulus;
use super::patterns::Uniform;
use crate::impl_stimulus;
use crate::visual::color::IntoRawRgba;
use crate::visual::{
    geometry::ToVertices,
    window::WindowState, Window,
};
use derive_more::Deref;

#[derive(Clone, Debug, Deref)]
pub struct ColorStimulus {
    _inner: PatternStimulus<Uniform>,
}

impl ColorStimulus {
    /// Create a new color stimulus. This is composed of a pattern stimulus with a uniform color pattern.
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        color: impl IntoRawRgba,
    ) -> Self {
        ColorStimulus {
            _inner: PatternStimulus::new_from_pattern(window, shape, Uniform::new(color)),
        }
    }

    /// Set the color of the pattern.
    pub fn set_color(&mut self, _color: impl IntoRawRgba) -> () {
        todo!()
    }
}

impl_stimulus!(ColorStimulus, PatternStimulus<Uniform>);
