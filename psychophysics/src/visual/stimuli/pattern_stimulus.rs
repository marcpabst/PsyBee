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

pub trait FillPattern: Send + Sync {
    /// Get the uniform buffer(s) data for the pattern. Every item in the vector should be the data for a single uniform buffer.
    ///
    /// The length of the returned vector must be equal to number of structs returned by `get_uniform_structs_code`.
    ///
    /// Be careful, WGSL expect very specific alignment for the data in the uniform buffer!
    fn get_uniform_buffers_data(&self, window: &Window) -> Vec<Vec<u8>>;

    /// Get the uniform struct(s) code for the pattern. This needs to be a vector of (String, String, String), matching the number of uniform buffers.
    /// The first string in the tuple should be the code for the struct, the second string should be the name of the struct and the third string should be the name of the variable.
    ///
    /// Length of the returned vector must be equal to the number of the vector returned by `get_uniform_buffers_data`.
    ///
    /// You are free to choose the name of the struct and variable, but both must contain the unique identifier provided by the `uuid` parameter.
    ///
    /// For example, this could be the return value for a pattern that has a single uniform buffer with a struct containing a single field `alpha`:
    /// ```
    /// return vec![(format!("struct {uuid}_s1] {{ alpha: f32, }}".to_string(), "[uuid]_s1".to_string(), "[uuid]_0".to_string())];
    /// ```
    fn get_uniform_structs_code(
        &self,
        uuid: &str,
    ) -> Vec<(String, String, String)>;

    /// An arbitrary number of functions that will be available to the fragment shader.
    /// You are free to choose the name of the function, but it must contain the unique identifier provided by the `uuid` parameter.
    ///
    /// For example, could be returned by a pattern that has a single uniform buffer with a struct containing a single field `alpha`,
    /// where the function sets the alpha value of the color to the alpha value of the uniform buffer:
    /// ```
    /// return format!("fn {uuid}_set_alpha(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {{ return vec4<f32>(color.r, color.g, color.b, {uuid}_0.alpha); }}");
    /// ```
    fn get_pattern_functions_code(&self, uuid: &str) -> String;

    /// The expression that will be evaluated in the fragment shader. Must evaluate to a vec4<f32>.
    /// You may call any functions defined in `get_pattern_functions_code`. Following variables are in scope:
    /// - `x: f32` and `y: f32`: the position of the pixel
    /// - `color: vec4<f32>`: the current color of the pixel
    ///
    /// For example, this could be returned by a pattern that has a single uniform buffer with a struct containing a single field `alpha`:
    /// ```
    /// return format!("{uuid}_set_alpha(x, y, color)");
    fn get_pattern_expression(&self, uuid: &str) -> String;
}

impl<P: FillPattern> PatternStimulus<P> {
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        pattern: P,
    ) -> Self {
        let uniform_buffers_data = pattern.get_uniform_buffers_data(window);
        let uniform_buffers_data = uniform_buffers_data
            .iter()
            .map(|data| data.as_slice())
            .collect::<Vec<_>>();
        let uniform_buffers_data = uniform_buffers_data.as_slice();

        // build up the fragment shader code
        // first, randomly create a unique name for the param struct using only uppercase and lowercase letters
        let uuid = crate::utils::create_random_lowercase_string(10);

        // then, get the uniform struct code
        let uniform_struct_code = pattern.get_uniform_structs_code(&uuid);

        // then, get the pattern function code
        let pattern_function_code = pattern.get_pattern_functions_code(&uuid);

        // get the pattern expression
        let pattern_expression = pattern.get_pattern_expression(&uuid);

        // then, build the fragment shader code
        let mut fragment_shader_code = "".to_string();

        // add the code for the uniform structs
        for (i, (struct_code, struct_name, variable_name)) in
            uniform_struct_code.iter().enumerate()
        {
            fragment_shader_code.push_str(&format!(
                "
                {struct_code}

                @group(1) @binding({i})
                var<uniform> {variable_name}: {struct_name};

                "
            ));
        }

        // add the code for the pattern functions
        fragment_shader_code = format!(
            "
            {fragment_shader_code}

            {pattern_function_code}

            @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
                    let x = in.position_org.x;
                    let y = in.position_org.y;
                    let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
                    return {pattern_expression};
                }}
        "
        );

        println!("{}", fragment_shader_code);

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
        let uniform_buffers_data =
            self.pattern.get_uniform_buffers_data(window);
        let uniform_buffers_data = uniform_buffers_data
            .iter()
            .map(|data| data.as_slice())
            .collect::<Vec<_>>();
        let uniform_buffers_data = uniform_buffers_data.as_slice();

        self.base_stimulus
            .set_uniform_buffers(uniform_buffers_data, gpu_state);
        self.base_stimulus.prepare(window, window_state, gpu_state);
    }

    fn render(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> () {
        self.base_stimulus.render(enc, view);
    }
}
