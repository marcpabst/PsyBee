// Copyright (c) 2024 marc
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{
    super::geometry::{Rectangle, Size},
    super::window::Window,
    base::{BaseStimulus, BaseStimulusImplementation},
};
use crate::visual::geometry::ToVertices;
use image;

pub struct ImageStimulusImplementation {
    image: image::DynamicImage,
    shape: Rectangle,
    update: bool, // flag to indicate that the image has changed
}

/// A simple image stimulus.
pub type ImageStimulus = BaseStimulus<ImageStimulusImplementation>;

impl ImageStimulus {
    /// Create a new image stimulus.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to which the stimulus will be added.
    /// * `image` - The image to be displayed.
    ///
    /// # Returns
    ///
    /// A new image stimulus.
    pub fn new_pixel_size(
        window: &Window,
        image: image::DynamicImage,
    ) -> Self {
        // create a shape the size of the image
        let shape = Rectangle::new(
            -Size::Pixels(image.width() as f64 / 2.0),
            -Size::Pixels(image.height() as f64 / 2.0),
            Size::Pixels(image.width() as f64),
            Size::Pixels(image.height() as f64),
        );
        Self::_new(window, image, shape)
    }

    /// Create a new image stimulus with a rectangle shape.
    pub fn new(
        window: &Window,
        image: image::DynamicImage,
        shape: Rectangle,
    ) -> Self {
        Self::_new(window, image, shape)
    }

    /// Internal function to create a new image stimulus.
    fn _new(
        window: &Window,
        image: image::DynamicImage,
        shape: Rectangle,
    ) -> Self {
        let window = window.clone();
        window.clone().run_on_render_thread(|| async move {
            let window_state = window.get_window_state().await;
            BaseStimulus::create(
                &window,
                &window_state,
                ImageStimulusImplementation::new(image, shape),
            )
        })
    }

    /// Set the image to be displayed.
    ///
    /// # Arguments
    /// * `image` - The image to be displayed.
    pub fn set_image(&self, image: image::DynamicImage) {
        (self.stimulus_implementation.lock_blocking()).image = image;
        // set update flag
        (self.stimulus_implementation.lock_blocking()).update = true;
    }

    /// Set the rectangle used to display the image on the screen.
    ///
    /// # Arguments
    ///
    /// * `rect` - A rectangle that defines the position and size of the image.
    pub fn set_rectangle(&self, rect: Rectangle) {
        (self.stimulus_implementation.lock_blocking()).shape = rect;
    }
}

impl ImageStimulusImplementation {
    pub fn new(image: image::DynamicImage, shape: Rectangle) -> Self {
        Self {
            image,
            shape,
            update: true,
        }
    }
}

impl BaseStimulusImplementation for ImageStimulusImplementation {
    fn update(
        &mut self,
        _screen_width_mm: f64,
        _viewing_distance_mm: f64,
        _screen_width_px: u32,
        _screen_height_px: u32,
    ) -> (Option<&[u8]>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>)
    {
        // texture data (if update is true, otherwise None)
        let texture_data = if self.update {
            self.update = false;
            self.get_texture_data()
        } else {
            None
        };

        (None, None, texture_data)
    }

    fn get_fragment_shader_code(&self) -> String {
        "
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(texture, texture_sampler, in.tex_coords);
        }
        "
        .to_string()
    }

    fn get_texture_data(&self) -> Option<Vec<u8>> {
        // convert from rgba to bgra
        let texture_data: Vec<u8> = self
            .image
            .to_rgba8()
            .chunks_exact(4)
            .flat_map(|chunk| {
                [
                    chunk[2], // r
                    chunk[1], // g
                    chunk[0], // b
                    chunk[3], // a
                ]
            })
            .collect();

        Some(texture_data)
    }

    fn get_texture_size(&self) -> Option<wgpu::Extent3d> {
        Some(wgpu::Extent3d {
            width: self.image.width(),
            height: self.image.height(),
            depth_or_array_layers: 1,
        })
    }

    fn get_geometry(
        &self,
    ) -> Box<dyn crate::visual::geometry::ToVertices> {
        Box::new(self.shape.clone())
    }
}
