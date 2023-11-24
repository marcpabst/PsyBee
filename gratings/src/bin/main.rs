use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use web_time::{SystemTime};
use wgpu::util::DeviceExt;
use wgpu::{Adapter, Device, ShaderModule, Surface, TextureView};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};

const PI: f32 = std::f32::consts::PI;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GratingStimulusParams {
    phase: f32,
    frequency: f32,
}

struct StimulusState {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

// create Renderable trait
trait Renderable {
    fn render(&self, device: &mut Device, view: &TextureView) -> wgpu::CommandBuffer;
}

// define grating stimulus
struct GratingStimulus {
    params: GratingStimulusParams,
    state: StimulusState,
    shader: wgpu::ShaderModule,
}

impl Renderable for GratingStimulus {
    fn render(&self, device: &mut Device, view: &TextureView) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // update the stimulus buffer
            let params = GratingStimulusParams {
                phase: self.params.phase,
                frequency: self.params.frequency,
            };

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stimulus Buffer"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::COPY_SRC,
            });

            encoder.copy_buffer_to_buffer(&buffer, 0, &self.get_state().buffer, 0, 8);
        }
        {
            // update the stimulus buffer

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let bind_group = &self.get_state().bind_group;
            let render_pipeline = &self.get_state().pipeline;
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_pipeline(&render_pipeline);

            rpass.draw(0..6, 0..1);
        }

        encoder.finish()
    }
}

// constructor for GratingStimulus
impl GratingStimulus {
    fn get_state(&self) -> &StimulusState {
        &self.state
    }

    fn create_stimulus_state(
        device: &Device,
        surface: &Surface,
        adapter: &Adapter,
        shader: &ShaderModule,
        params: GratingStimulusParams,
    ) -> StimulusState {
        // add phase as a uniform for the fragment shader
        let stimulus_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stimulus Buffer"),
            contents: bytemuck::cast_slice(&[params]),
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

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        StimulusState {
            buffer: stimulus_buffer,
            bind_group: stimulus_bind_group,
            pipeline: render_pipeline,
        }
    }

    fn new(device: &Device, surface: &Surface, adapter: &Adapter) -> Self {
        // Load the shaders from disk
        let shader: ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let stim_params = GratingStimulusParams {
            phase: 0.0,
            frequency: 100.0,
        };

        Self {
            params: stim_params,
            state: GratingStimulus::create_stimulus_state(
                &device,
                &surface,
                &adapter,
                &shader,
                stim_params,
            ),
            shader: shader,
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // get system timestamp (not adapter.get_presentation_timestamp() which is the timestamp of the last frame)
    let mut last_time = SystemTime::now();
    
    let mut n_frame = 0;

    // Create the logical device and command queue
    let (mut device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let mut stim = GratingStimulus::new(&device, &surface, &adapter);

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {}
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // render the stimulus
        let enc = stim.render(&mut device, &view);

        let now = SystemTime::now();
        let frame_duration = now.duration_since(last_time).unwrap().as_millis();

        queue.submit(Some(enc));
        frame.present();

        // print time since last frame
        if frame_duration != 16 {
            println!("Time since last frame: {:?} ms", frame_duration);
        }
        last_time = now;

        if n_frame % 8 == 0 {
            stim.params.phase = stim.params.phase + PI;
        }

        // update frame count
        n_frame = n_frame + 1;
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // get monitor
    let monitor = window.available_monitors().nth(1).unwrap_or_else(|| {
        println!("No second monitor found, using current monitor");
        window.current_monitor().unwrap()
    });

    // get video mode with biggest width
    let target_size = monitor
        .video_modes()
        .max_by_key(|m| m.size().width)
        .unwrap()
        .size();

    // get video mode with biggest width and highest refresh rate
    let video_mode = monitor
        .video_modes()
        .filter(|m| m.size() == target_size)
        .max_by_key(|m| m.refresh_rate_millihertz())
        .unwrap();

    // make fullscreen
    window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        println!("Running in wasm32");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
