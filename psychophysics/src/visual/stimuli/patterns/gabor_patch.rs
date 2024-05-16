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
        geometry::{Size, SizeVector2D, ToPixels},
        Window,
    },
};

/// A Gabor patch pattern
#[derive(Clone, Debug)]
pub struct GaborPatch {
    phase: f32,
    cycle_length: Size,
    color: RawRgba,
    mu: SizeVector2D,
    sigma: SizeVector2D,
}

impl GaborPatch {
    pub fn new<L, C, M, S>(phase: f32, cycle_length: L, color: C, mu: M, sigma: S) -> Self
    where
        L: Into<Size>,
        C: IntoRawRgba,
        M: Into<SizeVector2D>,
        S: Into<SizeVector2D>,
    {
        Self {
            phase,
            cycle_length: cycle_length.into(),
            color: color.convert_to_raw_rgba(ColorFormat::SRGBA8),
            mu: mu.into(),
            sigma: sigma.into(),
        }
    }
}

impl FillPattern for GaborPatch {
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

        let mu = self.mu.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );
        let mu = (mu.0 as f32, mu.1 as f32);

        let sigma = self.sigma.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );
        let sigma = (sigma.0 as f32, sigma.1 as f32);

        // return the uniform buffer data as a byte slice
        let data1 = [
            self.phase.to_ne_bytes(),
            cycle_length.to_ne_bytes(),
            mu.0.to_ne_bytes(),
            mu.1.to_ne_bytes(),
            sigma.0.to_ne_bytes(),
            sigma.1.to_ne_bytes(),
        ]
        .concat();
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
            mu: vec2<f32>,
            sigma: vec2<f32>,
            color: vec4<f32>,
        };
        
        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;

        // the 2D Gaussian function
        fn gaussian(x: f32, y: f32, mu: vec2<f32>, sigma: vec2<f32>) -> f32 {
            let normalizer = 1.0 / (2.0 * 3.14159265359 * sigma.x * sigma.y);
            let exponent = -0.5 * ((x - mu.x) * (x - mu.x) / (sigma.x * sigma.x) + (y - mu.y) * (y - mu.y) / (sigma.y * sigma.y));

            return normalizer * exp(exponent);
        }
        
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4f {
            let frequency = 1.0 / uniforms.cycle_length;
            let pos = vec4<f32>(in.position.xy, 0., 0.);
            var a = sin(frequency * pos.x + uniforms.phase);
            // modulate the alpha value with a 2D Gaussian
            let alpha_max = gaussian(0.0, 0.0, uniforms.mu, uniforms.sigma);
            let alpha = gaussian(pos.x, pos.y, uniforms.mu, uniforms.sigma) / alpha_max;
            return vec4<f32>(1.0 * a, 1.0 * a, 1.0 * a, alpha);
        }
        "
        .to_string()
    }
}
