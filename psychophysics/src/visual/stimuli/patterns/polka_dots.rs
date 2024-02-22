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

/// A Polka dot pattern
#[derive(Clone, Debug)]
pub struct PolkaDots {
    phase: (f32, f32),
    dot_size: Size,
    dot_distance: Size,
    dot_color: RawRgba,
    background_color: RawRgba,
}

impl PolkaDots {
    pub fn new<L, C, B>(
        phase: (f32, f32),
        dot_size: L,
        dot_distance: L,
        dot_color: C,
        background_color: B,
    ) -> Self
    where
        L: Into<Size>,
        C: IntoRawRgba,
        B: IntoRawRgba,
    {
        Self {
            phase,
            dot_size: dot_size.into(),
            dot_distance: dot_distance.into(),
            dot_color: dot_color.convert_to_raw_rgba(ColorFormat::SRGBA8),
            background_color: background_color.convert_to_raw_rgba(ColorFormat::SRGBA8),
        }
    }
}
impl FillPattern for PolkaDots {
    fn uniform_buffer_data(&self, window: &Window) -> Option<Vec<u8>> {
        let screen_width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let screen_width_px = window.width_px.load_relaxed();
        let screen_height_px = window.height_px.load_relaxed();

        // turn dot_size and dot_distance into floats (in pixels)
        let dot_size: f32 = self.dot_size.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;

        let dot_distance: f32 = self.dot_distance.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;

        // return the uniform buffer data as a byte slice
        let data1 = [
            self.phase.0.to_ne_bytes(),
            self.phase.1.to_ne_bytes(),
            dot_size.to_ne_bytes(),
            dot_distance.to_ne_bytes(),
        ]
        .concat();

        let data2 = self.dot_color.to_ne_bytes().to_vec();
        let data3 = self.background_color.to_ne_bytes().to_vec();

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
            dotSize: f32,
            dotDistance: f32,
            dot_color: vec4<f32>,
            background_color: vec4<f32>,
        };

        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            // Calculate the normalized coordinates of the fragment.
            let normalizedCoords = vec2<f32>(in.position.x, in.position.y);
            // Calculate the grid position.
            let gridPos = normalizedCoords / uniforms.dotDistance;
            // Find the nearest grid center.
            let nearestCenter = round(gridPos) * uniforms.dotDistance;
            // Calculate the distance from the nearest grid center.
            let distance = length(normalizedCoords - nearestCenter);
            // Determine if the fragment is within the dot size.
            let color = select(uniforms.dot_color, uniforms.background_color, distance > uniforms.dotSize / 2.0);
        
            // Set the fragment color.
            return color;
        }
        "
        .to_string()
    }
}
