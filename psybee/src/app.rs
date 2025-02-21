use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use pyo3::{
    pyclass, pyfunction,
    types::{PyDict, PyTuple},
    Py, PyAny, Python,
};
use renderer::wgpu::TextureFormat;
use wgpu::MemoryHints;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    monitor::MonitorHandle,
    window::{Window as WinitWindow, WindowId},
};

use crate::{
    errors,
    experiment::{EventLoopAction, ExperimentManager, Monitor},
    input::Event,
    visual::window::{PhysicalScreen, Window, WindowState},
    EventTryFrom,
};

type ArcMutex<T> = Arc<Mutex<T>>;

#[derive(Debug)]
pub struct GPUState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Debug)]
pub struct App {
    pub windows: Vec<Window>,
    pub gpu_state: ArcMutex<GPUState>,
    pub action_receiver: Receiver<EventLoopAction>,
    pub action_sender: Sender<EventLoopAction>,
    pub dummy_window: Option<Window>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (action_sender, action_receiver) = std::sync::mpsc::channel();

        let backend = wgpu::Backends::DX12 | wgpu::Backends::METAL;
        let instance_desc = wgpu::InstanceDescriptor {
            backends: backend,
            // use defaults for the rest
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_desc);

        // request an adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None, // idealy we would use the surface here, but we don't have it yet
        }))
        .expect("Failed to find an suitable graphics adapter. This is likely a bug, please report it.");

        log::debug!("Selected graphics adapter: {:?}", adapter.get_info());

        let mut limits = wgpu::Limits::downlevel_defaults();
        limits.max_storage_buffers_per_shader_stage = 16;

        let features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;

        // Create the logical device and command queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features,
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: limits.using_resolution(adapter.limits()),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to create device. This is likely a bug, please report it.");

        let gpu_state = GPUState {
            instance,
            adapter,
            device,
            queue,
        };

        Self {
            windows: vec![],
            gpu_state: Arc::new(Mutex::new(gpu_state)),
            action_receiver,
            action_sender,
            dummy_window: None,
        }
    }

    /// Create a new window with the given options.
    pub fn create_window(
        &self,
        // window_options: &WindowOptions,
        event_loop: &ActiveEventLoop,
    ) -> Window {
        let window_attributes = WinitWindow::default_attributes()
            .with_title("Winit window")
            .with_transparent(false);

        let winit_window = event_loop.create_window(window_attributes).unwrap();

        // make sure cursor is visible (for normlisation across platforms)
        winit_window.set_cursor_visible(true);

        winit_window.focus_window();

        // log::debug!("Window created: {:?}", winit_window);

        let winit_window = Arc::new(winit_window);

        let gpu_state = self.gpu_state.lock().unwrap();

        let instance = &gpu_state.instance;
        let adapter = &gpu_state.adapter;
        let device = &gpu_state.device;
        let queue = &gpu_state.queue;

        log::debug!("Creating wgup surface...");

        let surface = instance
            .create_surface(winit_window.clone())
            .expect("Failed to create surface. This is likely a bug, please report it.");

        // print supported swapchain formats
        let swapchain_formats = surface.get_capabilities(adapter).formats;
        log::debug!("Supported swapchain formats: {:?}", swapchain_formats);

        let size = winit_window.inner_size();

        let _swapchain_formats = adapter.get_texture_format_features(TextureFormat::Bgra8Unorm);

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = TextureFormat::Bgra8Unorm;
        let swapchain_view_format = vec![TextureFormat::Bgra8Unorm];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: swapchain_view_format,
            desired_maximum_frame_latency: 2,
        };

        log::debug!("Surface configuration: {:?}", config);

        surface.configure(device, &config);

        // set fullscreen mode
        // winit_window.set_fullscreen(fullscreen_mode);

        let wgpu_renderer = pollster::block_on(renderer::wgpu_renderer::WgpuRenderer::new(
            winit_window.clone(),
            instance,
            device,
            queue,
            swapchain_format,
        ));

        // create the renderer
        let mut renderer = renderer::DynamicRenderer::new(
            renderer::Backend::Skia,
            adapter,
            device,
            queue,
            swapchain_format,
            size.width,
            size.height,
        );

        let winit_id = winit_window.id();

        // set width of the screen to 30 cm
        let width_mm = 300.0;
        let viewing_distance = 1000.0;

        // create a pwindow
        let window_state = WindowState {
            winit_window: winit_window.clone(),
            surface,
            config,
            renderer,
            wgpu_renderer,
            mouse_cursor_visible: true,
            mouse_position: None,
            size: size.into(),
            physical_screen: PhysicalScreen::new(size.width, width_mm, viewing_distance),
            event_handlers: HashMap::new(), // TODO this should be a weak reference
        };

        // create channel for physical input
        let (mut event_broadcast_sender, physical_input_receiver) = async_broadcast::broadcast(10_000);
        event_broadcast_sender.set_overflow(true);
        // deactivate the receiver
        let event_broadcast_receiver = physical_input_receiver.deactivate();

        // create handle
        let window = Window {
            winit_id,
            state: Arc::new(Mutex::new(window_state)),
            gpu_state: self.gpu_state.clone(),
            event_broadcast_sender,
            event_broadcast_receiver,
        };

        let win_clone = window.clone();

        // TODO: add event handlers
        // add a default event handler for mouse move events, which updates the mouse
        // position
        // window.add_event_handler(EventKind::CursorMoved, move |event| {
        //     if let Some(pos) = event.position() {
        //         win_clone.inner().mouse_position = Some(pos.clone());
        //     };
        //     false
        // });

        window
    }

    // /// Run the app
    // pub fn run(&mut self) {
    //     // create event loop
    //     let event_loop = EventLoop::new().unwrap();
    //     event_loop.set_control_flow(ControlFlow::Poll);
    //     let _ = event_loop.run_app(self);
    // }

    /// Starts the experiment. This will block until the experiment.
    ///
    /// # Arguments
    ///
    /// * `experiment_fn` - The function that is your experiment. This function
    ///   will be called with a `Window` object that you can use to create
    ///   stimuli and submit frames to the window.
    pub fn run_experiment<F>(&mut self, experiment_fn: F)
    where
        F: FnOnce(ExperimentManager) -> Result<(), errors::PsybeeError> + 'static + Send,
    {
        log::debug!("Main task is running on thread {:?}", std::thread::current().id());

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let event_loop_proxy = event_loop.create_proxy();
        let event_loop_proxy2 = event_loop.create_proxy();

        let action_sender = self.action_sender.clone();

        // start experiment
        thread::spawn(move || {
            let event_manager = ExperimentManager::new(event_loop_proxy, action_sender);

            let res = experiment_fn(event_manager);

            // panic if the experiment function returns an error
            if let Err(e) = res {
                // throw error
                log::error!("Experiment failed with {:?}: {:}", e, e);
                // quit program
                std::process::exit(1);
            }
        });

        // start event loop
        let _ = event_loop.run_app(self);
    }

    // Start a thread that will dispath
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        println!("resumed");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        println!("user event");
        // check if we need to create a new window
        self.action_receiver.try_recv().map(|action| match action {
            EventLoopAction::CreateNewWindow(options, sender) => {
                let window = self.create_window(event_loop);
                self.windows.push(window.clone());
                sender.send(window).unwrap();
            }
            EventLoopAction::GetAvailableMonitors(sender) => {
                println!("getting monitors");
                let monitors = event_loop.available_monitors();

                // convert into a vector of monitors
                let monitors: Vec<Monitor> = monitors
                    .map(|monitor| {
                        Monitor::new(monitor.name().unwrap_or("Unnamed monitor".to_string()), (0, 0), monitor)
                    })
                    .collect();

                println!("sending monitors");
                sender.send(monitors).unwrap();
                println!("sent monitors");
            }
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // for now, exit the program
                std::process::exit(0);
                // find the window
                let window = self.windows.iter().find(|w| w.winit_id == window_id);

                if let Some(window) = window {
                    // remove the window
                    self.windows.retain(|w| w.winit_id != window_id);
                }
            }
            WindowEvent::Resized(size) => {
                // find the window
                let window = self.windows.iter().find(|w| w.winit_id == window_id);

                if let Some(window) = window {
                    // update the window size
                    window.resize(size);
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event: _,
                is_synthetic,
            } => {
                // find the window
                let window = self.windows.iter().find(|w| w.winit_id == window_id);

                if let Some(window) = window {
                    if let Some(input) = Event::try_from_winit(event.clone(), &window).ok() {
                        // if escape key was pressed, close window
                        if input.key_pressed("\u{1b}") {
                            // for now, just exit the program
                            std::process::exit(0);
                        }

                        // broadcast the event
                        window.event_broadcast_sender.try_broadcast(input.clone()).unwrap();

                        // send the event to the window
                        window.dispatch_event(input);
                    }
                }
            }
            _ => {}
        }
    }
}
