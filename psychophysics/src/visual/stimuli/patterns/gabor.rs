// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


use super::super::pattern_stimulus::FillPattern;
use crate::{
    utils::AtomicExt,
    visual::{
        color::{ColorFormat, IntoRawRgba, RawRgba},
        geometry::{Size, ToPixels},
        Window,
    },
};

/// A Sinosoidal grating pattern
#[derive(Clone, Debug)]
pub struct Gabor {
    pub phase: f32,
    pub cycle_length: Size,
    pub std_x: Size,
    pub std_y: Size,
    pub orientation: f32,
    pub color: RawRgba,
}

impl Gabor {
    pub fn new<L, C, M, N>(
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
        Self {
            phase,
            cycle_length: cycle_length.into(),
            std_x: std_x.into(),
            std_y: std_y.into(),
            orientation,
            color: color.convert_to_raw_rgba(ColorFormat::SRGBA8),
        }
    }

    pub fn set_phase(&mut self, phase: f32) -> () {
        self.phase = phase;
    }

    pub fn set_std_x<L>(&mut self, std_x: L) -> ()
    where
        L: Into<Size>,
    {
        self.std_x = std_x.into();
    }

    pub fn set_std_y<L>(&mut self, std_y: L) -> ()
    where
        L: Into<Size>,
    {
        self.std_y = std_y.into();
    }

    pub fn set_std<L>(&mut self, std: L) -> ()
    where
        L: Into<Size>,
    {
        let std = std.into();
        self.std_x = std.clone();
        self.std_y = std;
    }

    pub fn set_orientation(&mut self, orientation: f32) -> () {
        self.orientation = orientation;
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
    fn uniform_buffer_data(&mut self, window: &Window) -> Option<Vec<u8>> {
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

        // turn std_x into a float (in pixels)
        let std_x: f32 = self.std_x.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;

        // turn std_y into a float (in pixels)
        let std_y: f32 = self.std_y.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;

        // return the uniform buffer data as a byte slice
        let data1 = [
            self.phase.to_ne_bytes(),
            cycle_length.to_ne_bytes(),
            std_x.to_ne_bytes(),
            std_y.to_ne_bytes(),
            self.orientation.to_ne_bytes(),
        ]
        .concat();

        let data2 = self.color.to_ne_bytes().to_vec();

        // pad to 48 bytes
        let padding = vec![0; 48 - data1.len() - data2.len()];

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
            std_x: f32,
            std_y: f32,
            orientation: f32,
            color: vec4<f32>,
        };
        
        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;

        fn grating(x: f32, y: f32, orientation: f32, phase: f32, frequency: f32) -> f32 {
            return sin(2.0 * 3.141592653589793 * frequency * (x * cos(orientation) + y * sin(orientation)) + phase);
        }

        fn gaussian(x: f32, y: f32, x0: f32, y0: f32, sigma_x: f32, sigma_y: f32) -> f32 {
            return  exp(-((x - x0) * (x - x0) / (2.0 * sigma_x * sigma_x) + (y - y0) * (y - y0) / (2.0 * sigma_y * sigma_y)));
        }
        
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4f {
            let frequency = 1.0 / uniforms.cycle_length;
            let pos = vec4<f32>(in.position.xy, 0., 0.);
            var alpha = grating(pos.x, pos.y, uniforms.orientation, uniforms.phase, frequency);

            // apply gaussian envelope
            alpha = alpha * gaussian(pos.x, pos.y, 0.0, 0.0, uniforms.std_x, uniforms.std_y);

            return vec4<f32>(1.0 * alpha, 1.0 * alpha, 1.0 * alpha, 1.0);
        }
        "
        .to_string()
    }
}
