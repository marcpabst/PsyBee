// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use image::{DynamicImage, GenericImageView};

use super::super::pattern_stimulus::FillPattern;
use crate::{
    prelude::PsychophysicsError,
    utils::AtomicExt,
    visual::{
        color::{ColorFormat, IntoRawRgba, RawRgba},
        geometry::{Size, SizeVector2D, ToPixels},
        Window,
    },
};

/// A Sinosoidal grating pattern
#[derive(Clone, Debug)]
pub struct Gabor {
    pub phase: f32,
    pub cycle_length: Size,
    pub color: RawRgba,
}

impl Gabor {
    pub fn new<L, C>(phase: f32, cycle_length: L, color: C) -> Self
    where
        L: Into<Size>,
        C: IntoRawRgba,
    {
        Self {
            phase,
            cycle_length: cycle_length.into(),
            color: color.convert_to_raw_rgba(ColorFormat::SRGBA8),
        }
    }

    pub fn set_phase(&mut self, phase: f32) -> () {
        self.phase = phase;
    }

    pub fn set_color(&mut self, color: impl IntoRawRgba) -> () {
        self.color = color.convert_to_raw_rgba(ColorFormat::SRGBA8);
    }

    pub fn set_cycle_length<L>(&mut self, cycle_length: L) -> ()
    where
        L: Into<Size>,
    {
        self.cycle_length = cycle_length.into();
    }
}

impl FillPattern for Gabor {
    fn uniform_buffer_data(&self, window: &Window) -> Option<Vec<u8>> {
        let screen_width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let screen_width_px = window.width_px.load_relaxed();
        let screen_height_px = window.height_px.load_relaxed();

        // turn cycle_length into a float (in pixels)
        let cycle_length: f32 = self.cycle_length.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;

        // return the uniform buffer data as a byte slice
        let data1 = [self.phase.to_ne_bytes(), cycle_length.to_ne_bytes()].concat();
        let data2 = self.color.to_ne_bytes().to_vec();
        // 8 bytes of padding to align the data with 32 bytes
        let padding = vec![0; 8];
        Some([data1, data2, padding].concat())
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        struct Uniforms {
            phase: f32,
            cycle_length: f32,
            color: vec4<f32>,
        };
        
        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;
        
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4f {
            let frequency = 1.0 / uniforms.cycle_length;
            let pos = vec4<f32>(in.position.xy, 0., 0.);
            var alpha = sin(frequency * pos.x + uniforms.phase);
            return vec4<f32>(1.0 * alpha, 1.0 * alpha, 1.0 * alpha, 1.0);
        }
        "
        .to_string()
    }
}
