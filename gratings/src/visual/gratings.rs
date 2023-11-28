use super::shape::{ShapeParams, ShapeShader, ShapeStimulus};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
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

// make GratingsStimulus a alias for ShapeStimulus with GratingsShader and GratingsParams
pub type GratingsStimulus = ShapeStimulus<GratingsShader, GratingsParams>;

// implement new() for GratingsStimulus
impl GratingsStimulus {
    pub fn new(
        device: &Device,
        surface: &Surface,
        adapter: &Adapter,
        frequency: f32,
        phase: f32,
    ) -> Self {
        let shader = GratingsShader::new(device, phase, frequency);
        let params = GratingsParams { phase, frequency };

        let stim: ShapeStimulus<GratingsShader, GratingsParams> =
            ShapeStimulus::create(device, surface, adapter, shader, params);
        stim
    }
}
