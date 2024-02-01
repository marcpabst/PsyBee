// Copyright (c) 2024 marc
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{
    super::super::visual::color::*,
    super::geometry::ToVertices,
    super::window::Window,
    base::{BaseStimulus, BaseStimulusImplementation},
};
use bytemuck::{Pod, Zeroable};

pub struct ShapeStimulusImplementation {
    color: RawRgba,
    color_format: ColorFormat,
    shape: Box<dyn ToVertices>,
    params: ShapeStimulusParams,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShapeStimulusParams {
    color: RawRgba,
}

/// A simple shape stimulus.
pub type ShapeStimulus = BaseStimulus<ShapeStimulusImplementation>;

impl ShapeStimulus {
    /// Create a new gratings stimulus.
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        color: impl palette::IntoColor<
            palette::Xyza<palette::white_point::D65, f32>,
        >,
    ) -> Self {
        let window = window.clone();
        let color_format = window.color_format;
        let color = color_format.convert_to_raw_rgba(color);

        window.clone().run_on_render_thread(move || async move {
            // get window state
            let window_state = window.get_window_state_blocking();
            // convert color to raw rgba

            // create parameters
            let params = ShapeStimulusParams { color };

            let implementation = ShapeStimulusImplementation::new(
                color,
                color_format,
                params,
                shape,
            );

            BaseStimulus::create(
                &window,
                &window_state,
                implementation,
            )
        })
    }

    /// Set the color of the stimulus.
    pub fn set_color(
        &self,
        color: impl palette::IntoColor<
            palette::Xyza<palette::white_point::D65, f32>,
        >,
    ) {
        let _color = (self.stimulus_implementation.lock_blocking())
            .color_format
            .convert_to_raw_rgba(color);
        (self.stimulus_implementation.lock_blocking()).color = _color;
    }
}

impl ShapeStimulusImplementation {
    pub fn new(
        color: RawRgba,
        color_format: ColorFormat,
        params: ShapeStimulusParams,
        shape: impl ToVertices + 'static,
    ) -> Self {
        Self {
            color,
            color_format,
            params,
            shape: Box::new(shape),
        }
    }
}

impl BaseStimulusImplementation for ShapeStimulusImplementation {
    fn update(
        &mut self,
        _screen_width_mm: f64,
        _viewing_distance_mm: f64,
        _screen_width_px: u32,
        _screen_height_px: u32,
    ) -> (Option<&[u8]>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>)
    {
        // update color
        self.params.color = self.color;

        // return updated parameters
        (Some(bytemuck::bytes_of(&self.params)), None, None)
    }

    fn get_uniform_buffer_data(&self) -> Option<&[u8]> {
        // we need to convert the data to bytes
        Some(bytemuck::bytes_of(&self.params))
    }

    fn get_geometry(&self) -> Box<dyn ToVertices> {
        self.shape.clone_box()
    }

    fn get_fragment_shader_code(&self) -> String {
        "
        struct ShapeStimulusParams {
            color: vec4<f32>,
        };
        
        @group(0) @binding(0)
        var<uniform> params: ShapeStimulusParams;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return params.color;
        }"
        .to_string()
    }
}
