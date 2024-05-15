// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync::{Arc, Mutex};

use crate::visual::geometry::Transformable;
use crate::visual::window::WindowState;
use crate::visual::{geometry::ToVertices, Window};
use crate::GPUState;

use super::base_stimulus::BaseStimulus;
use super::Stimulus;

#[derive(Clone, Debug)]
pub struct PatternStimulus<P> {
    base_stimulus: BaseStimulus,
    pub pattern: Arc<Mutex<P>>,
}

#[macro_export]
macro_rules! generate_assessors {
    // Generate a setter and getter for a single field
    ($name:ident, $field:ident, Into<$type:ty>) => {
        // Generate the setter
        paste::paste! {
            pub fn [<set_ $field>]<P>(&mut self, $field: P) -> ()
            where
                P: Into<$type>,
            {
                self.$name.lock().unwrap().$field = $field.into();
            }
        }

        // Generate the getter
        pub fn $field(&self) -> $type {
            self.$name.lock().unwrap().$field.clone()
        }
    };

    ($name:ident, $field:ident, $type:ty) => {
        // Generate the setter
        paste::paste! {
            pub fn [<set_ $field>](&mut self, $field: $type) -> () {
                self.$name.lock().unwrap().$field = $field;
            }
        }

        // Generate the getter
        pub fn $field(&self) -> $type {
            self.$name.lock().unwrap().$field.clone()
        }
    };
}

pub trait FillPattern: Send + Sync {
    /// The shader language that the pattern uses. WGSL by default.
    const SHADER_LANGUAGE: &'static str = "wgsl";

    /// Get the fragment shader code for the pattern.
    ///
    /// You can bind one uniform buffer to the pattern and one texture. They will be automatically rebound to suitable bind groups.
    ///
    /// # Example
    ///
    /// ```
    /// fn fragment_shader_code(&self, window: &Window) -> String {
    ///    "struct Uniforms {{
    ///       color: vec4<f32>;
    ///   }};
    ///
    ///  @group(1) @binding(0)
    /// var<uniform> uniforms: Uniforms;
    ///
    /// @fragment
    /// fn main() -> @location(0) vec4f {
    ///   return uniforms.color;
    /// }}
    /// ```
    ///
    fn fragment_shader_code(&self, window: &Window) -> String;

    /// Get the uniform buffer size (in bytes) for the pattern. This is the size of the buffer that will be passed to the fragment shader.
    ///
    /// If the pattern does not use a uniform buffer, this function should return `None`.
    fn uniform_buffer_size(&mut self, window: &Window) -> Option<usize> {
        // by default, return the size of self.uniform_buffer_data()
        // this is slightly inefficient, but it's the best we can do without knowing the actual data
        if let Some(data) = self.uniform_buffer_data(window) {
            Some(data.len())
        } else {
            None
        }
    }

    /// Get the uniform buffer data for the pattern. This is the data that will be passed to the fragment shader.
    /// If the buffer did not change since the last time this function was called, it can return `None`.
    ///
    /// If the pattern does not use a uniform buffer, this function should return `None`.
    ///
    /// Must match the size returned by `uniform_buffer_size`. Also, make sure that you honor WGPU's alignment requirements.
    fn uniform_buffer_data(&mut self, _window: &Window) -> Option<Vec<u8>> {
        None
    }

    /// Returns the current uniform buffer data for the pattern. As opposed to `uniform_buffer_data`,
    /// this function should return `None` if the buffer did not change since the last time this function was called.
    fn updated_uniform_buffers_data(&mut self, window: &Window) -> Option<Vec<u8>> {
        self.uniform_buffer_data(window)
    }

    /// Returns the extent of the texture that will be used by the pattern.
    /// This is optional and can be used to provide a texture to the pattern.
    ///
    /// If the pattern does not use a texture, this function should return `None`.
    fn texture_extent(&self, _window: &Window) -> Option<wgpu::Extent3d> {
        None
    }

    /// Get the texture data for the pattern. This is optional and can be used to provide a texture to the pattern.
    ///
    /// If the pattern does not use a texture, this function should return `None`.
    fn texture_data(&self, _window: &Window) -> Option<Vec<u8>> {
        None
    }

    /// Returns the current texture data for the pattern. As opposed to `texture_data`,
    /// this function should return `None` if the texture did not change since the last time this function was called.
    fn updated_texture_data(&self, window: &Window) -> Option<Vec<u8>> {
        self.texture_data(window)
    }
}

impl<P: FillPattern> PatternStimulus<P> {
    pub fn new_from_pattern(
        window: &Window,
        geometry: impl ToVertices + 'static,
        mut pattern: P,
    ) -> Self {
        // get the uniform buffer data
        let _uniform_buffer_size = pattern.uniform_buffer_size(window);

        let uniform_buffer_data =
            if let Some(uniform_buffer_data) = pattern.uniform_buffer_data(window) {
                uniform_buffer_data
            } else {
                // return an empty buffer
                vec![]
            };

        let texture_size = pattern.texture_extent(window);
        let texture_data = pattern.texture_data(window);
        let fragment_shader_code = pattern.fragment_shader_code(window);

        Self {
            base_stimulus: BaseStimulus::new(
                window,
                geometry,
                &fragment_shader_code,
                texture_size,
                texture_data,
                &[uniform_buffer_data],
            ),
            pattern: Arc::new(Mutex::new(pattern)),
        }
    }

    pub fn set_pattern(&mut self, pattern: P) -> () {
        self.pattern = Arc::new(Mutex::new(pattern));
    }
}

impl<P: FillPattern> std::ops::Deref for PatternStimulus<P> {
    type Target = BaseStimulus;
    fn deref(&self) -> &Self::Target {
        &self.base_stimulus
    }
}

impl<P: FillPattern> Stimulus for PatternStimulus<P> {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) -> () {
        // update the uniform buffer
        let mut pattern = self.pattern.lock().unwrap();
        if let Some(uniform_buffer_data) = pattern.updated_uniform_buffers_data(window) {
            self.base_stimulus
                .set_uniform_buffers(&[uniform_buffer_data.as_slice()], gpu_state);
        }

        // update the texture

        if let Some(texture_data) = pattern.updated_texture_data(window) {
            self.base_stimulus.set_texture(texture_data, gpu_state);
        }

        self.base_stimulus.prepare(window, window_state, gpu_state);
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.base_stimulus.render(enc, view);
    }
}

impl<T> Transformable for PatternStimulus<T> {
    fn set_transformation(
        &self,
        transformation: crate::visual::geometry::Transformation2D,
    ) {
        // set the transformation of the base stimulus
        self.base_stimulus.set_transformation(transformation);
    }

    fn add_transformation(
        &self,
        transformation: crate::visual::geometry::Transformation2D,
    ) {
        // add the transformation to the base stimulus
        self.base_stimulus.add_transformation(transformation);
    }
}
