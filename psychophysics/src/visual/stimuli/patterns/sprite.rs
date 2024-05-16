// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use image::{DynamicImage, GenericImageView};

use super::super::pattern_stimulus::FillPattern;
use crate::{
    prelude::PsychophysicsError,
    visual::{
        Window,
    },
};

/// A Sprite is like an image, but it can hold multiple images of the same size on the GPU and switch between them
/// by changing the texture index.
#[derive(Clone, Debug)]
pub struct Sprite {
    // The images that the sprite holds
    images: Vec<image::DynamicImage>,
    /// The index of the current image (0-based, does not wrap around)
    current_index: u64,
    /// Frames per second of the sprite (None for manual control)
    fps: Option<f64>,
    /// The number of times the sprite should repeat (None for infinite)
    repeat: Option<u64>,
    /// Time the frame was initialized
    init_time: std::time::Instant,
}

impl std::fmt::Display for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sprite with {} images", self.images.len())
    }
}

impl Sprite {
    /// Create a new sprite from a list of images. All images must have the same dimensions, otherwise this function will return an error.
    pub fn new(
        images: Vec<image::DynamicImage>,
        fps: Option<f64>,
        repeat: Option<u64>,
    ) -> Result<Self, PsychophysicsError> {
        // check that the vector is not empty
        if images.is_empty() {
            return Err(PsychophysicsError::EmptyVectorError);
        }
        // check if there is only one image (this is currently not supported)
        if images.len() == 1 {
            return Err(PsychophysicsError::SingleImageError);
        }
        // check that all images have the same dimensions
        let dimensions = images[0].dimensions();
        if images.iter().all(|image| image.dimensions() == dimensions) {
            Ok(Self {
                images,
                current_index: 0,
                fps: fps,
                repeat: repeat,
                init_time: std::time::Instant::now(),
            })
        } else {
            Err(PsychophysicsError::NonIdenticalDimensionsError(
                dimensions.0,
                dimensions.1,
            ))
        }
    }

    /// Create a new sprite from a list of image paths. All images must have the same dimensions, otherwise this function will return an error.
    pub fn new_from_paths(
        paths: Vec<&str>,
        fps: Option<f64>,
        repeat: Option<u64>,
    ) -> Result<Self, PsychophysicsError> {
        let images = paths
            .iter()
            .map(|path| image::open(path))
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(images, fps, repeat)
    }

    /// Create a new sprite from a spritesheet. The sprite sheet must contain images of the same size.
    pub fn new_from_spritesheet(
        path: &str,
        num_sprites_x: u32,
        num_sprites_y: u32,
        fps: Option<f64>,
        repeat: Option<u64>,
    ) -> Result<Self, PsychophysicsError> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();

        // check that the sprite sheet is divisible by the sprite size
        if width % num_sprites_x != 0 || height % num_sprites_y != 0 {
            return Err(PsychophysicsError::WrongDimensionsError(
                num_sprites_x,
                num_sprites_y,
                width,
                height,
            ));
        }

        // calculate the sprite size
        let sprite_width = width / num_sprites_x;
        let sprite_height = height / num_sprites_y;

        // split the image into sprites
        let mut images: Vec<image::DynamicImage> = Vec::new();
        for y in (0..height).step_by(sprite_height as usize) {
            for x in (0..width).step_by(sprite_width as usize) {
                let sprite = image.view(x, y, sprite_width, sprite_height).to_image();
                images.push(DynamicImage::ImageRgba8(sprite));
            }
        }

        Self::new(images, fps, repeat)
    }

    /// Set the current texture index of the sprite.
    pub fn set_image_index(&mut self, index: u64) {
        self.current_index = index;
    }

    /// Move to the next image in the sprite.
    pub fn advance_image_index(&mut self) {
        self.current_index += 1;
    }

    // Reset the sprite.
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.init_time = std::time::Instant::now();
    }
}

impl FillPattern for Sprite {
    fn texture_extent(&self, _window: &Window) -> Option<wgpu::Extent3d> {
        let (width, height) = self.images[0].dimensions();
        Some(wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: self.images.len() as u32,
        })
    }

    fn texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        for image in &self.images {
            let d = image.to_rgba8().to_vec();
            data.extend_from_slice(&d);
        }
        log::debug!("Sprite texture data length: {}", data.len());
        Some(data)
    }

    fn updated_texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        return None;
    }

    fn uniform_buffer_data(&mut self, _window: &Window) -> Option<Vec<u8>> {
        // if fps is set, calculate the index based on the time
        let mut index = self.current_index;
        if let Some(fps) = self.fps {
            let elapsed = self.init_time.elapsed().as_secs_f64();
            let frames = elapsed * fps;
            index = frames as u64;
        }

        if let Some(repeat) = self.repeat {
            // if repeat is set, make sure that the new index is within the maximum index
            // if not, set index to the last image
            if index >= self.images.len() as u64 * repeat {
                index = self.images.len() as u64 * repeat - 1;
            }
        }

        // calculate the current index by wrapping around the number of images
        let wrapped_index = index % self.images.len() as u64;

        Some(wrapped_index.to_ne_bytes().to_vec())
    }

    fn updated_uniform_buffers_data(&mut self, window: &Window) -> Option<Vec<u8>> {
        self.uniform_buffer_data(window)
    }

    fn fragment_shader_code(&self, _window: &Window) -> String {
        "
        struct VertexOutput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        };

        struct Uniforms {
            texture_index: u32,
        };

        @group(0) @binding(1)
        var texture_array: texture_2d_array<f32>;

        @group(0) @binding(2)
        var texture_sampler: sampler;

        @group(1) @binding(0)
        var<uniform> uniforms: Uniforms;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            var o = textureSample(texture_array, texture_sampler, in.tex_coords, uniforms.texture_index);
            // fom rgba to bgra
            return vec4<f32>(o.b, o.g, o.r, o.a);
        }
        "
        .to_string()
    }
}
