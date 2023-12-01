pub mod gratings;
pub mod shape;
pub mod text;

use std::{
    mem,
    sync::{Arc, Mutex},
};

use futures::executor::block_on;

use web_time::SystemTime;
use wgpu::{Device, Queue, SurfaceConfiguration};
use winit::event_loop::EventLoop;

// Renderable trait should be implemented by all visual stimuli
// the API is extremely simple: render() and update() and follows the
// the middlewares pattern used by wgpu
pub trait Renderable {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> ();
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
}

pub struct Screen<F>
where
    F: FnMut(&mut Frame),
{
    render_func: F,
    frame: Option<Frame>, // the current frame
}

impl<F> Screen<F>
where
    F: FnMut(&mut Frame),
{
    pub fn new(render_func: F) -> Self {
        Self {
            render_func: render_func, // the render function
            frame: None,              // the current frame
        }
    }
}
impl<F> Renderable for Screen<F>
where
    F: FnMut(&mut Frame),
{
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // check if we have a frame
        if self.frame.is_some() {
            // get the frame
            let frame = self.frame.as_mut().unwrap();
            frame.render(enc, view);
            // consume the frame
            self.frame = None;
        }
    }

    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> () {
        // create a new frame
        let mut frame = Frame::new();
        // call the render function
        (self.render_func)(&mut frame);
        // prepare the frame
        frame.prepare(device, queue, view, config);
        // assign the frame to self
        self.frame = Some(frame);
    }
}

pub struct Frame {
    renderables: Vec<Box<dyn Renderable>>,
}

impl Frame {
    // create a new frame
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
        }
    }
    // add a renderable to the frame
    pub fn add(&mut self, renderable: &(impl Renderable + Clone + 'static)) -> () {
        let renderable = Box::new(renderable.clone());
        self.renderables.push(renderable);
    }
}

impl Renderable for Frame {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> () {
        // call prepare() on all renderables
        for renderable in &mut self.renderables {
            renderable.prepare(device, queue, view, config);
        }
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // call render() on all renderables
        for renderable in &mut self.renderables {
            renderable.render(enc, view);
        }
    }
}

pub struct Window {
    pub window: Arc<Mutex<winit::window::Window>>,
    event_loop: Option<EventLoop<()>>,
    pub device: Arc<Mutex<wgpu::Device>>,
    pub instance: Arc<Mutex<wgpu::Instance>>,
    pub adapter: Arc<Mutex<wgpu::Adapter>>,
    pub queue: Arc<Mutex<wgpu::Queue>>,
    pub surface: Arc<Mutex<wgpu::Surface>>,
    pub config: Arc<Mutex<wgpu::SurfaceConfiguration>>,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
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
        let size = window.inner_size();

        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (mut device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        Self {
            window: Arc::new(Mutex::new(window)),
            event_loop: Some(event_loop),
            device: Arc::new(Mutex::new(device)),
            instance: Arc::new(Mutex::new(instance)),
            adapter: Arc::new(Mutex::new(adapter)),
            queue: Arc::new(Mutex::new(queue)),
            surface: Arc::new(Mutex::new(surface)),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn show(&mut self, mut renderable: impl Renderable + 'static) -> () {
        let config = self.config.clone();
        let device = self.device.clone();
        let queue = self.queue.clone();
        let surface = self.surface.clone();
        let window = self.window.clone();
        let adapter = self.adapter.clone();
        let instance = self.instance.clone();
        let mut event_loop = self.event_loop.take().unwrap();
        {
            event_loop.run(move |event, _, control_flow| {
                // Have the closure take ownership of the resources.
                // `event_loop.run` never returns, therefore we must do this to ensure
                // the resources are properly cleaned up.
                let _ = (&instance, &adapter);

                *control_flow = winit::event_loop::ControlFlow::Poll;
                match event {
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::Resized(size),
                        ..
                    } => {
                        // Reconfigure the surface with the new size
                        config.lock().unwrap().width = size.width;
                        config.lock().unwrap().height = size.height;
                        surface
                            .lock()
                            .unwrap()
                            .configure(&device.lock().unwrap(), &config.lock().unwrap());
                        // On macos the window needs to be redrawn manually after resizing
                        window.lock().unwrap().request_redraw();
                    }
                    winit::event::Event::RedrawRequested(_) => {}
                    winit::event::Event::WindowEvent {
                        event:
                            winit::event::WindowEvent::KeyboardInput {
                                device_id: _,
                                input: _,
                                is_synthetic: _,
                            },
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    _ => {}
                }

                let frame = surface
                    .lock()
                    .unwrap()
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // create a command encoder
                let mut encoder = device.lock().unwrap().create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    },
                );

                // prepare the renderable
                renderable.prepare(
                    &device.lock().unwrap(),
                    &queue.lock().unwrap(),
                    &view,
                    &config.lock().unwrap(),
                );

                // // create a render pass for the surface
                // let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                //     label: None,
                //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                //         view: &view,
                //         resolve_target: None,
                //         ops: wgpu::Operations {
                //             load: wgpu::LoadOp::Load,
                //             store: wgpu::StoreOp::Store,
                //         },
                //     })],
                //     depth_stencil_attachment: None,
                //     timestamp_writes: None,
                //     occlusion_query_set: None,
                // });

                // render the renderable
                renderable.render(&mut encoder, &view);

                queue.lock().unwrap().submit(Some(encoder.finish()));
                frame.present();
            });
        }
    }
}
