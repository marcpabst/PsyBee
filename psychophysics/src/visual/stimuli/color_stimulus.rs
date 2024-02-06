// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::{
    visual::{
        color::RawRgba, geometry::ToVertices, stimuli::base_stimulus::BaseStimulus,
        window::WindowState, Window,
    },
    GPUState,
};
use derive_more::Deref;

/// A stimulus that displays a single color.
#[derive(Clone, Debug, Deref)]
pub struct ColorStimulus {
    #[deref]
    base_stimulus: BaseStimulus,
    pub color: RawRgba,
}

const FRAGMENT_SHADER: &str = "
            struct ShapeStimulusParams {
                color: vec4<f32>,
            }

            @group(1) @binding(0)
            var<uniform> params: ShapeStimulusParams;

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return params.color;
            }";

impl ColorStimulus {
    /// Create a new color stimulus.
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        color: impl palette::IntoColor<palette::Xyza<palette::white_point::D65, f32>>,
    ) -> Self {
        let color_format = window.color_format;
        let color = color_format.convert_to_raw_rgba(color);

        let uniform_buffer_data = bytemuck::bytes_of(&color);

        Self {
            base_stimulus: BaseStimulus::new(
                window,
                shape,
                FRAGMENT_SHADER,
                None,
                &[uniform_buffer_data],
            ),
            color,
        }
    }

    /// Set the color of the stimulus.
    pub fn set_color(
        &mut self,
        color: impl palette::IntoColor<palette::Xyza<palette::white_point::D65, f32>>,
    ) -> () {
        todo!()
    }
}

impl super::Stimulus for ColorStimulus {
    fn prepare(&mut self, window: &Window, window_state: &WindowState, gpu_state: &GPUState) -> () {
        self.base_stimulus.prepare(window, window_state, gpu_state);
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.base_stimulus.render(enc, view);
    }
}
