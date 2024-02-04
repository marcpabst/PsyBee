// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use palette::white_point::A;
use rand::Rng;

// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::visual::window::WindowState;
use crate::visual::{geometry::ToVertices, Window};
use crate::GPUState;

use super::base_stimulus::BaseStimulus;
use super::Stimulus;

#[derive(Clone, Debug)]
pub struct PatternStimulus<P> {
    base_stimulus: BaseStimulus,
    pattern: P,
}

pub trait Pattern: Send + Sync {
    fn get_uniform_buffers_data(&self, window: &Window) -> Vec<Vec<u8>>;
    fn get_uniform_struct_code(&self, struct_name: &str) -> String;
    fn get_pattern_function_code(&self, variable_name: &str) -> String;
}

impl<P: Pattern> PatternStimulus<P> {
    pub fn new(window: &Window, shape: impl ToVertices + 'static, pattern: P) -> Self {
        let uniform_buffers_data = pattern.get_uniform_buffers_data(window);
        let uniform_buffers_data = uniform_buffers_data
            .iter()
            .map(|data| data.as_slice())
            .collect::<Vec<_>>();
        let uniform_buffers_data = uniform_buffers_data.as_slice();

        // build up the fragment shader code
        // first, randomly create a unique name for the param struct using only uppercase and lowercase letters
        let unique_id = crate::utils::create_random_lowercase_string(10);

        let struct_name = "PatternParams".to_string() + &unique_id;
        let variable_name = "params_".to_string() + &unique_id;

        // then, get the uniform struct code
        let uniform_struct_code = pattern.get_uniform_struct_code(&struct_name);

        // then, get the pattern function code
        let pattern_function_code = pattern.get_pattern_function_code(&variable_name);

        // then, build the fragment shader code
        let fragment_shader_code = format!(
            "
            // uniform struct
            {}

            // bind (uniforms are always bound to group 1)
            @group(1) @binding(0)
            var<uniform> {}: {};

            // fragment shader
            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
            {}
            }}
            ",
            uniform_struct_code, variable_name, struct_name, pattern_function_code
        );

        Self {
            base_stimulus: BaseStimulus::new(
                window,
                shape,
                &fragment_shader_code,
                None,
                uniform_buffers_data,
            ),
            pattern: pattern,
        }
    }

    pub fn set_pattern(&mut self, pattern: P) -> () {
        self.pattern = pattern;
    }
}

impl<P: Pattern> std::ops::Deref for PatternStimulus<P> {
    type Target = BaseStimulus;
    fn deref(&self) -> &Self::Target {
        &self.base_stimulus
    }
}

impl<P: Pattern> Stimulus for PatternStimulus<P> {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) -> () {
        let uniform_buffers_data = self.pattern.get_uniform_buffers_data(window);
        let uniform_buffers_data = uniform_buffers_data
            .iter()
            .map(|data| data.as_slice())
            .collect::<Vec<_>>();
        let uniform_buffers_data = uniform_buffers_data.as_slice();

        self.base_stimulus
            .set_uniform_buffers(uniform_buffers_data, gpu_state);
        self.base_stimulus.prepare(window, window_state, gpu_state);
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.base_stimulus.render(enc, view);
    }
}
