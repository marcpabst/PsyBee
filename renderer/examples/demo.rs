extern crate renderer;

use renderer::affine::Affine;
use renderer::bitmaps::DynamicBitmap;
use renderer::brushes::{ColorStop, Extend, Gradient, GradientKind, ImageSampling};
use renderer::prelude::{FillStyle, ImageFitMode};
use renderer::styles::StrokeStyle;
use renderer::wgpu::{
    BindGroup, Buffer, Device, Queue, RenderPipeline, Surface, Texture, TextureFormat, TextureUsages,
};
use renderer::{Backend, DynamicRenderer};
use std::error::Error;
use std::sync::Arc;
use vello::RendererOptions;
use wgpu::util::DeviceExt;
use wgpu::InstanceDescriptor;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_format: TextureFormat,
    render_pipeline: RenderPipeline,
    renderer: DynamicRenderer,
    texture: Texture,
    gamma_buffer: Buffer,
    bind_group: BindGroup,
    window: Arc<Window>,
    size: winit::dpi::PhysicalSize<u32>,
    start_time: std::time::Instant,
    image: DynamicBitmap,
}

struct App {
    state: Option<State>,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GammaParams {
    r: [f32; 8],
    g: [f32; 8],
    b: [f32; 8],
    correction: u32,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
                None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let (width, height) = (size.width, size.height);

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[1];

        let renderer = DynamicRenderer::new(Backend::Skia, &device, surface_format, width, height);

        // create a render pipeline
        let render_pipeline = Self::create_render_pipelie(&device, surface_format);
        let texture = Self::create_texture(&device, width, height);
        let gamma_buffer = Self::create_uniform_buffer(&device);
        let bind_group = Self::create_bind_group(&device, &texture);

        // load image from file
        let image = image::load_from_memory(include_bytes!("assets/images/dog.png")).unwrap();
        let bitmap = renderer.create_bitmap(image);

        Self {
            device,
            queue,
            surface,
            surface_format,
            render_pipeline,
            renderer,
            texture,
            gamma_buffer,
            bind_group,
            window,
            size,
            start_time: std::time::Instant::now(),
            image: bitmap,
        }
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn render(&mut self) {
        let device = &self.device;
        let queue = &self.queue;
        let surface = &self.surface;

        // create scene
        let mut scene = self.renderer.create_scene(self.size.width, self.size.height);

        // get the elapsed time
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let offset = elapsed.sin() * 100.0 + 200.0;

        let mut lattice = Affine::scale_xy(20.0, 10.0);
        lattice.pre_skew(0.5, 0.0);

        scene.draw_shape_fill(
            renderer::shapes::Shape::rectangle((0.0, 0.0), 1200.0, 1200.0),
            renderer::brushes::Brush::ShapePattern {
                shape: renderer::shapes::Shape::rectangle((0.0, 0.0), 10.0, 10.0),
                latice: lattice,
                brush: &renderer::brushes::Brush::Solid(renderer::colors::RGBA::new(1.0, 1.0, 0.0, 1.0)),
            },
            None,
            None,
        );

        scene.draw_shape_fill(
            renderer::shapes::Shape::rounded_rectangle((100.0, 100.0), 900.0, 900.0, offset as f64),
            renderer::brushes::Brush::Gradient(Gradient {
                extend: Extend::Reflect,
                kind: GradientKind::Radial {
                    center: (0.0, 0.0).into(),
                    radius: 600.0,
                },
                stops: vec![
                    ColorStop {
                        offset: 0.0,
                        color: renderer::colors::RGBA::new(1.0, 0.0, 0.0, 1.0),
                    },
                    ColorStop {
                        offset: 0.5,
                        color: renderer::colors::RGBA::new(0.0, 1.0, 0.0, 1.0),
                    },
                    ColorStop {
                        offset: 1.0,
                        color: renderer::colors::RGBA::new(0.0, 0.0, 1.0, 1.0),
                    },
                ],
            }),
            None,
            None,
        );

        scene.start_layer(
            renderer::styles::BlendMode::Modulate,
            renderer::shapes::Shape::rectangle((0.0, 0.0), 1200.0, 1200.0),
            Default::default(),
            None,
            1.0,
        );

        // the mask
        scene.draw_shape_fill(
            renderer::shapes::Shape::rectangle((000.0, 000.0), 1200.0, 1200.0),
            renderer::brushes::Brush::Image {
                image: &self.image,
                start: (0.0, 0.0).into(),
                fit_mode: ImageFitMode::Exact {
                    width: 100.0,
                    height: 100.0,
                },
                edge_mode: Extend::Repeat.into(),
                sampling: ImageSampling::Linear,
                transform: Some(Affine::scale((offset / 20.0) as f64)),
                alpha: None,
            },
            None,
            None,
        );

        scene.end_layer();
        // scene.end_layer();

        scene.draw_shape_fill(
            renderer::shapes::Shape::ellipse((600.0, 600.0), 200.0, 600.0, offset as f64),
            renderer::brushes::Brush::Solid(renderer::colors::RGBA::new(1.0, 1.0, 1.0, 1.0)),
            None,
            None,
        );

        scene.draw_shape_fill(
            renderer::shapes::Shape::circle((600.0, 600.0), offset as f64),
            renderer::brushes::Brush::Solid(renderer::colors::RGBA::new(1.0, 1.0, 0.0, 1.0)),
            None,
            None,
        );

        scene.draw_shape_stroke(
            renderer::shapes::Shape::circle((600.0, 600.0), offset as f64),
            renderer::brushes::Brush::Solid(renderer::colors::RGBA::new(0.0, 0.0, 1.0, 1.0)),
            StrokeStyle::new(10.0),
            None,
            None,
        );

        scene.draw_shape_fill(
            renderer::shapes::Shape::circle((600.0, 600.0), 100.0),
            renderer::brushes::Brush::Solid(renderer::colors::RGBA::new(0.0, 1.0, 0.0, 1.0)),
            None,
            None,
        );

        // render the scene
        self.renderer
            .render_to_texture(device, queue, &self.texture, scene.width(), scene.height(), &mut scene);

        // create a new render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // create a new surface texture
        let surface_texture = surface.get_current_texture().unwrap();

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            // bind the render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
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

        // present the surface
        surface_texture.present();
    }

    /// Re-size the texture
    pub fn resize(&mut self, width: u32, height: u32) {
        let device = &self.device;
        self.size = winit::dpi::PhysicalSize::new(width, height);
        self.texture = Self::create_texture(device, width, height);
        self.bind_group = Self::create_bind_group(device, &self.texture);
        self.configure_surface();
        println!("Resized to {:?}", (width, height));
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
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
                                correction: 3, // 0: none, 1: psychopy, 2: polylog4, 3: polylog5, 4: polylog6
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
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/shaders/render.wgsl").into()),
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
}

impl App {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Window {
        let window_attributes = Window::default_attributes()
            .with_title("Winit window")
            .with_transparent(false);

        let window = event_loop.create_window(window_attributes).unwrap();

        window
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // create a window
        let window = self.create_window(event_loop);
        // create a new state
        let state = pollster::block_on(State::new(Arc::new(window)));
        // configure the surface
        state.configure_surface();
        // set the state
        self.state = Some(state);

        println!("Resumed");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always folloed up by redraw request.
                state.resize(size.width, size.height);
            }
            _ => (),
        }
    }
}
fn main() {
    // create an application
    let mut app = App { state: None };
    // run the application
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run_app(&mut app);
}
