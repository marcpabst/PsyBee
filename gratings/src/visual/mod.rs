pub mod gratings;
pub mod shape;
pub mod text;

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

// Renderable trait should be implemented by all visual stimuli
// the API is extremely simple: render() and update() and follows the
// the middlewares pattern used by wgpu
pub trait Renderable {
    fn render<'pass>(&'pass self, device: &mut Device, pass: &mut RenderPass<'pass>) -> ();
    fn update(
        &mut self,
        device: &mut Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        config: &SurfaceConfiguration,
    ) -> ();
}
