use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::visual::Renderable;
use bytemuck::{Pod, Zeroable};

use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, CommandBuffer, CommandEncoder, Device, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, Surface, SurfaceConfiguration,
};
pub trait ShapeShader<P: ShapeParams> {
    fn update(&self, params: &mut P);
    fn get_shader(&self) -> &ShaderModule;
}
pub trait ShapeParams: Pod + Zeroable + Copy {}

#[derive(Debug)]
// define gratings stimulus
pub struct ShapeStimulus<S: ShapeShader<P>, P: ShapeParams> {
    buffer: Arc<Mutex<wgpu::Buffer>>,
    bind_group: Arc<Mutex<wgpu::BindGroup>>,
    pipeline: Arc<Mutex<wgpu::RenderPipeline>>,
    pub params: Arc<Mutex<P>>,
    pub shader: Arc<Mutex<S>>,
}

impl<S: ShapeShader<P>, P: ShapeParams> Clone for ShapeStimulus<S, P> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            bind_group: self.bind_group.clone(),
            pipeline: self.pipeline.clone(),
            params: self.params.clone(),
            shader: self.shader.clone(),
        }
    }
}

impl<S: ShapeShader<P>, P: ShapeParams> Renderable for ShapeStimulus<S, P> {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> () {
        // call the shader update function
        self.shader
            .lock()
            .unwrap()
            .update(&mut self.params.lock().unwrap());

        // update the stimulus buffer
        queue.write_buffer(
            self.buffer.lock().unwrap().deref(),
            0,
            bytemuck::cast_slice(&[*self.params.lock().unwrap()]),
        );
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        let pipeline = self.pipeline.lock().unwrap();
        let bind_group = self.bind_group.lock().unwrap();
        {
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&pipeline.deref());
            rpass.set_bind_group(0, &bind_group.deref(), &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}

impl<S: ShapeShader<P>, P: ShapeParams> ShapeStimulus<S, P> {
    pub fn create(
        device: &Device,
        surface: &Surface,
        adapter: &Adapter,
        shader: S,
        stim_params: P,
    ) -> Self {
        // add phase as a uniform for the fragment shader
        let stimulus_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stimulus Buffer"),
            contents: bytemuck::cast_slice(&[stim_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let stimulus_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("stimulus_bind_group_layout"),
            });

        let stimulus_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &stimulus_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: stimulus_buffer.as_entire_binding(),
            }],
            label: Some("stimulus_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&stimulus_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader.get_shader(),
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader.get_shader(),
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            buffer: Arc::new(Mutex::new(stimulus_buffer)),
            bind_group: Arc::new(Mutex::new(stimulus_bind_group)),
            pipeline: Arc::new(Mutex::new(render_pipeline)),
            shader: Arc::new(Mutex::new(shader)),
            params: Arc::new(Mutex::new(stim_params)),
        }
    }
}
