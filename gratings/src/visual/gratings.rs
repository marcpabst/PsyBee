use super::shape::{ShapeParams, ShapeShader, ShapeStimulus};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, ops::Deref};
use wgpu::{Adapter, Device, ShaderModule, Surface};

// the parameters for the gratings stimulus, these will be used as uniforms
// and made available to the shader
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GratingsParams {
    pub phase: f32,
    pub frequency: f32,
}

impl ShapeParams for GratingsParams {}

pub struct GratingsShader {
    shader: ShaderModule,
}

pub type GratingsStimulus = ShapeStimulus<GratingsShader, GratingsParams>;

impl GratingsStimulus {
    pub fn new(window: &super::Window, frequency: f32, phase: f32) -> Self {
        let binding = window.device.clone();
        let device = binding.lock().unwrap();
        let binding = window.surface.clone();
        let surface = binding.lock().unwrap();
        let binding = window.adapter.clone();
        let adapter = binding.lock().unwrap();

        let shader = GratingsShader::new(&device, phase, frequency);
        let params = GratingsParams { phase, frequency };

        ShapeStimulus::create(&device, &surface, &adapter, shader, params)
    }
}

impl GratingsShader {
    pub fn new(device: &Device, _phase: f32, _frequency: f32) -> Self {
        let shader: ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/gratings.wgsl"))),
        });

        Self { shader }
    }
}

impl ShapeShader<GratingsParams> for GratingsShader {
    fn update(&self, params: &mut GratingsParams) {
        // update the stimulus buffer
        let _params = GratingsParams {
            phase: params.phase,
            frequency: params.frequency,
        };
    }
    fn get_shader(&self) -> &ShaderModule {
        &self.shader
    }
}
