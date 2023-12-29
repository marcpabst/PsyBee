use super::{
    super::geometry::{Size, ToVertices, Transformation2D},
    super::pwindow::Window,
    base::{BaseStimulus, BaseStimulusImplementation, BaseStimulusParams},
};
use crate::utils::BlockingLock;
use bytemuck::{Pod, Zeroable};
use futures_lite::future::block_on;
use std::borrow::Cow;
use wgpu::{Device, ShaderModule};

pub enum GratingFunction {
    Sin,
    Custom(String),
}

impl GratingFunction {
    /// Returns valid WGSL code for the grating function.
    pub fn to_string(&self) -> String {
        match self {
            GratingFunction::Sin => {
                "sin(2.0 * PI * (x / cycle_length + phase))".to_string()
            }
            GratingFunction::Custom(s) => s.clone(),
        }
    }
}

/// The parameters for the gratings stimulus, these will be used as uniforms
/// and made available to the shader.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GratingsStimulusParams {
    pub phase: f32,
    pub cycle_length: f32, // in pixels
}

impl ToWGSL for GratingsStimulusParams {
    fn to_wgsl(&self) -> String {
        format!(concat!(
            "struct GratingsStimulusParams {{\n",
            "phase: f32;\n",
            "cycle_length: f32;\n",
            "}};\n"
        ))
    }
}

impl BaseStimulusParams for GratingsStimulusParams {}

pub struct GratingsShader {
    shader: ShaderModule,
    cycle_length: Size,
    phase: f32,
}

pub type GratingsStimulus<'a, G> =
    BaseStimulus<G, GratingsShader, GratingsStimulusParams>;

impl<G: ToVertices> GratingsStimulus<'_, G> {
    /// Create a new gratings stimulus.
    pub fn new(
        window_handle: &Window,
        shape: G,
        function_string: String,
        cycle_length: impl Into<Size>,
        phase: f32,
    ) -> Self {
        let window = window_handle.get_window_state_blocking();
        let device = &window.device;

        let shader = GratingsShader::new(&device, phase, cycle_length.into());

        let params = GratingsStimulusParams {
            phase,
            cycle_length: 0.0,
        };

        drop(window); // this prevent a deadlock (argh, i'll have to refactor this)

        BaseStimulus::create(window_handle, shader, shape, params, None)
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

impl GratingsShader {
    pub fn new(device: &Device, phase: f32, frequency: Size) -> Self {
        let shader: ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shaders/gratings.wgsl"
                ))),
            });

        Self {
            shader,
            cycle_length: frequency,
            phase: phase,
        }
    }
}

impl BaseStimulusImplementation<GratingsStimulusParams> for GratingsShader {
    fn update(
        &mut self,
        params: &mut GratingsStimulusParams,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Option<Vec<u8>> {
        // update the shader params
        params.cycle_length = self.cycle_length.to_pixels(
            width_mm as f64,
            viewing_distance_mm as f64,
            width_px,
            height_px,
        ) as f32;

        None
    }
    fn get_shader(&self) -> &ShaderModule {
        &self.shader
    }
}
