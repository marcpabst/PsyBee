use super::{
    pwindow::PWindow,
    shape::{ShapeParams, ShapeShader, ShapeStimulus},
};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::{Device, ShaderModule};

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
    pub fn new(window: &PWindow, frequency: f32, phase: f32) -> Self {
        let device = &window.device;
        let surface = &window.surface;
        let adapter = &window.adapter;

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
