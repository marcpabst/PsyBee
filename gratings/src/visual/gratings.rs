use super::shape::{ShapeParams, ShapeShader, ShapeStimulus};
use crate::visual::Renderable;
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use web_time::SystemTime;
use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, CommandEncoder, Device, MultisampleState, Queue, RenderPass, ShaderModule, Surface,
    SurfaceConfiguration, TextureFormat, TextureView,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GratingsParams {
    pub phase: f32,
    pub frequency: f32,
}

impl ShapeParams for GratingsParams {}

pub struct GratingsShader {
    pub shader: ShaderModule,
}

impl GratingsShader {
    pub fn new(device: &Device, phase: f32, frequency: f32) -> Self {
        let shader: ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/gratings.wgsl"))),
        });

        Self { shader: shader }
    }
}

impl ShapeShader<GratingsParams> for GratingsShader {
    fn update(&self, params: &mut GratingsParams) -> () {
        // update the stimulus buffer
        let params = GratingsParams {
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
        let params = GratingsParams {
            phase: phase,
            frequency,
        };

        let stim: ShapeStimulus<GratingsShader, GratingsParams> =
            ShapeStimulus::create(device, surface, adapter, shader, params);
        return stim;
    }
}
