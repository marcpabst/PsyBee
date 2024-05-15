// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use image::{DynamicImage, GenericImageView};

use super::super::pattern_stimulus::FillPattern;
use crate::visual::Window;

#[derive(Clone, Debug)]
pub struct Image {
    buffer: Vec<u8>,
    dimensions: (u32, u32),
    dirty: bool,
}

impl Image {
    pub fn new(image: image::DynamicImage) -> Self {
        Self {
            buffer: image.to_rgba8().to_vec(),
            dimensions: image.dimensions(),
            dirty: false,
        }
    }

    pub fn new_from_path(path: &str) -> Result<Self, image::ImageError> {
        log::debug!("Loading image from path: {}", path);
        let image = image::open(path)?;
        Ok(Self {
            buffer: image.to_rgba8().to_vec(),
            dimensions: image.dimensions(),
            dirty: true,
        })
    }

    pub fn set_image(&mut self, image: DynamicImage) {
        self.buffer = image.to_rgba8().to_vec();
        self.dirty = true;
    }

    pub fn set_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
        self.dirty = true;
    }
}

impl FillPattern for Image {
    fn texture_extent(&self, _window: &Window) -> Option<wgpu::Extent3d> {
        let (width, height) = self.dimensions;
        Some(wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        })
    }

    fn texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        Some(self.buffer.clone())
    }

    fn updated_texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        if self.dirty {
            None
        } else {
            None
        }
    }

    fn uniform_buffer_data(&mut self, _window: &Window) -> Option<Vec<u8>> {
        Some(vec![0; 32])
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        @group(0) @binding(1)
        var texture: texture_2d<f32>;

        @group(0) @binding(2)
        var texture_sampler: sampler;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            var o = vec4<f32>(textureSample(texture, texture_sampler, in.tex_coords));
            // fom rgba to bgra
            return vec4<f32>(o.b, o.g, o.r, o.a);
        }
        "
        .to_string()
    }
}
