use crate::visual::Renderable;
use bytemuck::{Pod, Zeroable};


use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, CommandEncoder, Device, Queue, RenderPass, ShaderModule, Surface,
    SurfaceConfiguration,
};


pub trait ShapeShader<P: ShapeParams> {
    fn update(&self, params: &mut P);
    fn get_shader(&self) -> &ShaderModule;
}

pub trait ShapeParams: Pod + Zeroable + Copy {}

// define gratings stimulus
pub struct ShapeStimulus<S: ShapeShader<P>, P: ShapeParams> {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    pub params: P,
    pub shader: S,
}

impl<S: ShapeShader<P>, P: ShapeParams> Renderable for ShapeStimulus<S, P> {
    fn render<'pass>(&'pass self, _device: &mut Device, pass: &mut RenderPass<'pass>) {
        {
            // update the stimulus buffer
            let bind_group = &self.bind_group;
            let render_pipeline = &self.pipeline;
            pass.set_pipeline(render_pipeline);
            pass.set_bind_group(0, bind_group, &[]);

            pass.draw(0..6, 0..1);
        }
    }
    fn update(
        &mut self,
        device: &mut Device,
        _queue: &Queue,
        encoder: &mut CommandEncoder,
        _config: &SurfaceConfiguration,
    ) {
        {
            // call the shader update function
            self.shader.update(&mut self.params);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stimulus Buffer"),
                contents: bytemuck::cast_slice(&[self.params]),
                usage: wgpu::BufferUsages::COPY_SRC,
            });

            encoder.copy_buffer_to_buffer(&buffer, 0, &self.buffer, 0, 8);
        }
    }
}

// constructor for GratingStimulus
// impl GratingStimulus {
//     fn get_state(&self) -> &StimulusState {
//         &self.state
//     }
// }
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
            buffer: stimulus_buffer,
            bind_group: stimulus_bind_group,
            pipeline: render_pipeline,
            shader,
            params: stim_params,
        }
    }
}
