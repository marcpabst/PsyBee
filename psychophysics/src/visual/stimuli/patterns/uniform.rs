// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


use super::super::pattern_stimulus::FillPattern;
use crate::{
    visual::{
        color::{ColorFormat, IntoRawRgba, RawRgba},
        Window,
    },
};

#[derive(Clone, Debug)]
pub struct Uniform {
    color: RawRgba,
}

impl Uniform {
    pub fn new(color: impl IntoRawRgba) -> Self {
        Self {
            color: color.convert_to_raw_rgba(ColorFormat::SRGBA8),
        }
    }

    pub fn set_color(&mut self, color: impl IntoRawRgba) {
        self.color = color.convert_to_raw_rgba(ColorFormat::SRGBA8);
    }
}

impl FillPattern for Uniform {
    fn uniform_buffer_data(&mut self, _window: &Window) -> Option<Vec<u8>> {
        let bytes = self.color.to_ne_bytes().to_vec();
        Some(bytes)
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        struct Uniforms {
            color: vec4<f32>,
        };
        
        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;
        
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4f {
            return uniforms.color;
        }
        "
        .to_string()
    }
}
