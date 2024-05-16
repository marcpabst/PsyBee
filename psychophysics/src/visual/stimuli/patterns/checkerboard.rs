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
        geometry::{SizeVector2D, ToPixels},
        Window,
    },
};

/// A Checkerboard pattern
#[derive(Clone, Debug)]
pub struct Checkerboard {
    phase: (f32, f32),
    cycle_length: SizeVector2D,
    color1: RawRgba,
    color2: RawRgba,
}

impl Checkerboard {
    pub fn new<L, C1, C2>(
        phase: (f32, f32),
        cycle_length: L,
        color1: C1,
        color2: C2,
    ) -> Self
    where
        L: Into<SizeVector2D>,
        C1: IntoRawRgba,
        C2: IntoRawRgba,
    {
        Self {
            phase,
            cycle_length: cycle_length.into(),
            color1: color1.convert_to_raw_rgba(ColorFormat::SRGBA8),
            color2: color2.convert_to_raw_rgba(ColorFormat::SRGBA8),
        }
    }
}

impl FillPattern for Checkerboard {
    fn uniform_buffer_data(&mut self, window: &Window) -> Option<Vec<u8>> {
        let screen_width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let screen_width_px = window.width_px.load_relaxed();
        let screen_height_px = window.height_px.load_relaxed();

        // turn cycle_length into a float (in pixels)
        let cycle_length = self.cycle_length.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        let cycle_length_x = cycle_length.0 as f32;
        let cycle_length_y = cycle_length.1 as f32;

        // return the uniform buffer data as a byte slice
        let data1 = [
            self.phase.0.to_ne_bytes(),
            self.phase.1.to_ne_bytes(),
            cycle_length_x.to_ne_bytes(),
            cycle_length_y.to_ne_bytes(),
        ]
        .concat();
        let data2 = self.color1.to_ne_bytes().to_vec();
        let data3 = self.color2.to_ne_bytes().to_vec();
        // 8 bytes of padding to align the data with 32 bytes
        let padding = vec![0; 8];
        Some([data1, data2, data3, padding].concat())
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        struct Uniforms {
            phase: vec2<f32>,
            cycle_length: vec2<f32>,
            color1: vec4<f32>,
            color2: vec4<f32>,
        };

        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            let adjusted_x = (in.position.x + uniforms.phase.x) / uniforms.cycle_length.x;
            let adjusted_y = (in.position.y + uniforms.phase.y) / uniforms.cycle_length.y;

            // use floor to get the integer part of the adjusted coordinates
            if (floor(adjusted_x) + floor(adjusted_y)) % 2 == 0 {
                return uniforms.color1;
            } else {
                return uniforms.color2;
            }
        }
        "
        .to_string()
    }
}
