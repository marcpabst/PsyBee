use super::{
    geometry::Triangulatable,
    pwindow::WindowHandle,
    shape::{ShapeParams, ShapeShader, ShapeStimulus},
};
use bytemuck::{Pod, Zeroable};
use futures_lite::future::block_on;
use std::borrow::Cow;
use wgpu::{Device, ShaderModule};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PlainParams {
    pub color: [f32; 4],
}

impl ShapeParams for PlainParams {}

pub struct PlainShader {
    shader: ShaderModule,
}

pub type PlainStimulus<G> = ShapeStimulus<G, PlainShader, PlainParams>;

impl<G: Triangulatable> GratingsStimulus<G> {
    pub fn new(window_handle: &WindowHandle, shape: G, frequency: f32, phase: f32) -> Self {
        let window = block_on(window_handle.get_window());
        let device = &window.device;
        let surface = &window.surface;
        let adapter = &window.adapter;

        let shader = PlainShader::new(&device, phase, frequency);
        let params = PlainParams { phase, frequency };

        ShapeStimulus::create(&device, &surface, &adapter, shader, shape, params)
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
