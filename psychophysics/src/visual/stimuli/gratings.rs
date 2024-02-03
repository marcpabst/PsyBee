// Copyright (c) 2024 marc
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{
    super::geometry::{Size, ToVertices},
    super::window::Window,
    base::{BaseStimulus, BaseStimulusImplementation},
};
use bytemuck::{Pod, Zeroable};

use std::sync::atomic::Ordering;
use wgpu::Device;

pub struct GratingsStimulusImplementation {
    cycle_length: Size,
    phase: f32,
    shape: Box<dyn ToVertices>,
    params: GratingsStimulusParams,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GratingsStimulusParams {
    phase: f32,
    cycle_length: f32,
}

/// A simple gratings stimulus.
pub type GratingsStimulus =
    BaseStimulus<GratingsStimulusImplementation>;

impl GratingsStimulus {
    /// Create a new gratings stimulus.
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        cycle_length: impl Into<Size>,
        _phase: f32,
    ) -> Self {
        let window = window.clone();
        let cycle_length: Size = cycle_length.into();

        // window.clone().run_on_render_thread(|| async move {
        let window_state = window.get_window_state_blocking();
        let gpu_state = window.get_gpu_state_blocking();
        let device = &gpu_state.device;
        let config = &window_state.config;

        // get screen size, viewing distance
        let screen_width_mm =
            window.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window.viewing_distance.load(Ordering::Relaxed);

        // get screen size in pixels
        let screen_width_px = config.width;
        let screen_height_px = config.height;

        // create parameters
        let params = GratingsStimulusParams {
            cycle_length: cycle_length.to_pixels(
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            ) as f32,

            phase: 0.0,
        };

        let implementation = GratingsStimulusImplementation::new(
            &device,
            0.0,
            cycle_length,
            params,
            shape,
        );

        BaseStimulus::create(&window, &window_state, implementation)
        //})
    }

    /// Set the length of a cycle.
    pub fn set_cycle_length(&self, length: impl Into<Size>) {
        (self.stimulus_implementation.lock_blocking()).cycle_length =
            length.into();
    }

    /// Set the phase.
    pub fn set_phase(&self, phase: f32) {
        (self.stimulus_implementation.lock_blocking()).phase = phase;
    }
}

impl GratingsStimulusImplementation {
    pub fn new(
        _device: &Device,
        phase: f32,
        cycle_length: Size,
        params: GratingsStimulusParams,
        shape: impl ToVertices + 'static,
    ) -> Self {
        Self {
            cycle_length,
            phase,
            shape: Box::new(shape),
            params,
        }
    }
}

impl BaseStimulusImplementation for GratingsStimulusImplementation {
    fn update(
        &mut self,
        _screen_width_mm: f64,
        _viewing_distance_mm: f64,
        _screen_width_px: u32,
        _screen_height_px: u32,
    ) -> (Option<&[u8]>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>)
    {
        // update phase
        self.params.phase = self.phase;

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
        struct GratingStimulusParams {
            phase: f32,
            cycle_length: f32,
        };
        
        @group(0) @binding(0)
        var<uniform> params: GratingStimulusParams;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            let frequency = 1.0 / params.cycle_length;
            let pos_org = vec4<f32>(in.position_org.xy, 0., 0.);
            var alpha = sin(frequency * pos_org.x + params.phase);
            return vec4<f32>(1.0 * alpha, 1.0 * alpha, 1.0 * alpha, 1.0);
        }"
        .to_string()
    }
}
