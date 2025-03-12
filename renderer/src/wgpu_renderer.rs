use std::sync::Arc;

use wgpu::{
    util::DeviceExt, BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, Surface, Texture, TextureFormat,
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GammaParams {
    r: [f32; 8],
    g: [f32; 8],
    b: [f32; 8],
    correction: u32,
}

pub struct WgpuRenderer {
    surface_format: TextureFormat,
    render_pipeline: RenderPipeline,
    texture: Texture,
    gamma_buffer: Buffer,
    bind_group: BindGroup,
    size: PhysicalSize<u32>,
}

impl WgpuRenderer {
    pub async fn new(
        window: Arc<Window>,
        _instance: &Instance,
        device: &Device,
        _queue: &Queue,
        surface_format: TextureFormat,
    ) -> Self {
        let size = window.inner_size();
        let (width, height) = (size.width, size.height);

        // create a render pipeline
        let render_pipeline = Self::create_render_pipelie(&device, surface_format);
        let texture = Self::create_texture(&device, width, height);
        let gamma_buffer = Self::create_uniform_buffer(&device);
        let bind_group = Self::create_bind_group(&device, &texture);

        Self {
            surface_format,
            render_pipeline,
            texture,
            gamma_buffer,
            bind_group,
            size,
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn configure_surface(&self, surface: &Surface, device: &Device) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(device, &surface_config);
    }

    /// Re-size the texture
    pub fn resize(&mut self, width: u32, height: u32, surface: &Surface, device: &Device) {
        self.size = winit::dpi::PhysicalSize::new(width, height);
        self.texture = Self::create_texture(device, width, height);
        self.bind_group = Self::create_bind_group(device, &self.texture);
        self.configure_surface(surface, device);
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba16Float],
        })
    }

    fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gamma Buffer"),
            size: std::mem::size_of::<GammaParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_bind_group(device: &wgpu::Device, texture: &wgpu::Texture) -> wgpu::BindGroup {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Gamma Buffer"),
                            contents: bytemuck::cast_slice(&[GammaParams {
                                correction: 0, // 0: none, 1: psychopy, 2: polylog4, 3: polylog5, 4: polylog6
                                r: [
                                    0.9972361456765942,
                                    0.5718201120693766,
                                    0.1494526003308258,
                                    0.021348959590415988,
                                    0.0016066519145011171,
                                    4.956890077371443e-05,
                                    0.0,
                                    0.0,
                                ],
                                g: [
                                    1.0058002029776596,
                                    0.5695706025327177,
                                    0.14551632725612368,
                                    0.020115266744271217,
                                    0.0014548822571441762,
                                    4.3086307473990124e-05,
                                    0.0,
                                    0.0,
                                ],
                                b: [
                                    1.0116733520722856,
                                    0.5329488652553003,
                                    0.11728724922990535,
                                    0.012259928984426039,
                                    0.000528402626505164,
                                    4.086604661837748e-06,
                                    0.0,
                                    0.0,
                                ],
                            }]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        }),
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }

    fn create_render_pipelie(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/render.wgsl").into()),
        });

        // create a bind group layout for texture and sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(&"vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(&"fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
        });

        render_pipeline
    }

    pub fn render_to_surface_and_present(&mut self, device: &Device, queue: &Queue, surface: &Surface) {
        // create a new surface texture
        let surface_texture = surface.get_current_texture().unwrap();

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_to_texture(device, queue, &surface_texture_view);

        // present the surface
        surface_texture.present();
    }

    pub fn render_to_texture(&mut self, device: &Device, queue: &Queue, texture_view: &wgpu::TextureView) {
        // create a new render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            // bind the render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // bind the render pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            // bind the bind group
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            // draw the quad
            render_pass.draw(0..6, 0..1);
        }

        // submit the render pass
        queue.submit(Some(encoder.finish()));
    }
}
