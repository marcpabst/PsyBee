// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::utils::AtomicExt;
use crate::visual::color::RawRgba;
use crate::visual::geometry::{Size, Vertex};
use crate::visual::window::WindowState;
use crate::visual::{
    geometry::{ToVertices, Transformation2D},
    Window,
};
use crate::GPUState;
use async_lock::Mutex;
use rand::distributions::uniform;
use std::sync::{atomic::AtomicUsize, Arc};
use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use super::base_stimulus::BaseStimulus;
use super::Stimulus;

#[derive(Clone, Debug)]
pub struct GratingsStimulus {
    base_stimulus: BaseStimulus,
    grating_type: GratingType,
}

#[derive(Clone, Debug)]
pub enum GratingType {
    Sine { phase: f32, cycle_length: Size },
}

impl GratingType {
    pub fn get_fragment_shader_code(&self) -> &'static str {
        match self {
            GratingType::Sine {
                phase: _,
                cycle_length: _,
            } => {
                "
                struct Params {
                    phase: f32,
                    cycle_length: f32,
                };
                
                @group(0) @binding(0)
                var<uniform> params: Params;
        
                @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                    let frequency = 1.0 / params.cycle_length;
                    let pos_org = vec4<f32>(in.position_org.xy, 0., 0.);
                    var alpha = sin(frequency * pos_org.x + params.phase);
                    return vec4<f32>(1.0 * alpha, 1.0 * alpha, 1.0 * alpha, 1.0);
                }
                "
            }
        }
    }

    pub fn get_uniform_buffer_data(&self, window: &Window) -> Vec<u8> {
        match self {
            GratingType::Sine {
                phase,
                cycle_length,
            } => {
                let screen_width_mm = window.physical_width.load_relaxed();
                let viewing_distance_mm = window.viewing_distance.load_relaxed();
                let screen_width_px = window.width_px.load_relaxed();
                let screen_height_px = window.height_px.load_relaxed();

                // turn cycle_length into a float (in pixels)
                let cycle_length: f32 = cycle_length.to_pixels(
                    screen_width_mm,
                    viewing_distance_mm,
                    screen_width_px,
                    screen_height_px,
                ) as f32;
                // return the uniform buffer data as a byte slice
                let data = [phase.to_ne_bytes(), cycle_length.to_ne_bytes()].concat();
                return data;
            }
        }
    }
}

impl GratingsStimulus {
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        grating_type: GratingType,
    ) -> Self {
        let uniform_buffer_data = grating_type.get_uniform_buffer_data(window);
        let uniform_buffer_data = Some(uniform_buffer_data.as_slice());
        let fragment_shader_code = grating_type.get_fragment_shader_code();

        Self {
            base_stimulus: BaseStimulus::new(
                window,
                shape,
                fragment_shader_code,
                None,
                uniform_buffer_data,
            ),
            grating_type,
        }
    }
}

impl std::ops::Deref for GratingsStimulus {
    type Target = BaseStimulus;
    fn deref(&self) -> &Self::Target {
        &self.base_stimulus
    }
}

impl Stimulus for GratingsStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) -> () {
        let uniform_buffer_data = self.grating_type.get_uniform_buffer_data(window);
        let uniform_buffer_data = uniform_buffer_data.as_slice();

        self.base_stimulus
            .set_uniform_buffer(uniform_buffer_data, gpu_state);
        self.base_stimulus.prepare(window, window_state, gpu_state);
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.base_stimulus.render(enc, view);
    }
}
