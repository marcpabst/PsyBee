#![allow(unused)]
#[macro_use]
use std::collections::HashMap;
use std::{
    pin::Pin,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use async_channel::{bounded, Receiver, Sender};
use derive_debug::Dbg;
use experiment::{py_run_experiment, ExperimentManager};
use futures_lite::{future::block_on, Future};
use pyo3::{prelude::*, py_run};
use renderer::wgpu_renderer;
use visual::geometry::Size;
use wgpu::{MemoryHints, TextureFormat};
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    monitor::VideoMode,
};

use crate::input::{Event, EventHandlingExt, EventKind, EventTryFrom};

pub mod app;
pub mod audio;
pub mod errors;
pub mod input;
pub mod utils;
pub mod visual;

pub mod experiment;

// re-export wgpu
pub use wgpu;

// types to make the code more readable
pub(crate) type RenderThreadChannelPayload = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

use std::thread;

use pyo3::types::{PyDict, PyList, PyTuple, PyType};

use crate::visual::window::Frame;

// macro that adds a sub-module to the current module
// example usage:
//
macro_rules! new_submodule {
    ($supermodule:ident, $supermodule_name:literal, $name:literal) => {{
        let m = PyModule::new($supermodule.py(), $name)?;
        m.setattr("__module__", concat!($supermodule_name, ".", $name))?;
        m.py()
            .import("sys")?
            .getattr("modules")?
            .set_item(concat!($supermodule_name, ".", $name), &m)?;
        $supermodule.add_submodule(&m)?;
        m
    }};
}

/// This module is implemented in Rust.
#[pymodule]
fn psydk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_run_experiment, m)?);
    m.add_class::<ExperimentManager>()?;

    let m_visual = {
        let m = new_submodule!(m, "psydk", "visual");

        let m_stimuli = {
            let m = new_submodule!(m, "psydk.visual", "stimuli");
            m.add_class::<visual::stimuli::PyStimulus>()?;
            m.add_class::<visual::stimuli::gabor::PyGaborStimulus>()?;
            m.add_class::<visual::stimuli::image::PyImageStimulus>()?;
            m.add_class::<visual::stimuli::shape::PyShapeStimulus>()?;
            m.add_class::<visual::stimuli::pattern::PyPatternStimulus>()?;
            m.add_class::<visual::stimuli::text::PyTextStimulus>()?;
            m
        };

        m.add_submodule(&m_stimuli)?;

        let m_geometry = {
            let m = new_submodule!(m, "psydk.visual", "geometry");
            m.add_class::<visual::geometry::Transformation2D>()?;
            m.add_class::<visual::geometry::Shape>()?;
            m.add_class::<visual::geometry::Size>()?;
            m
        };

        m.add_submodule(&m_geometry)?;

        let m_color = {
            let m = new_submodule!(m, "psydk.visual", "color");
            m.add_class::<visual::color::LinRgba>()?;
            m.add_function(wrap_pyfunction!(visual::color::py_rgb, &m)?)?;
            m.add_function(wrap_pyfunction!(visual::color::py_rgba, &m)?)?;
            m.add_function(wrap_pyfunction!(visual::color::py_linrgb, &m)?)?;
            m.add_function(wrap_pyfunction!(visual::color::py_linrgba, &m)?)?;
            m
        };

        m.add_submodule(&m_color)?;

        m
    };

    m.add_submodule(&m_visual)?;

    Ok(())
}

// pub trait FutureReturnTrait: Future<Output = ()> + 'static + Send {}
// impl<F> FutureReturnTrait for F where F: Future<Output = ()> + 'static + Send {}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[pyclass]
// pub struct Monitor {
//     #[pyo3(get, set)]
//     pub name: String,
//     #[pyo3(get, set)]
//     pub resolution: (u32, u32),
//     handle: winit::monitor::MonitorHandle,
// }

// #[pymethods]
// impl Monitor {
//     pub fn __repr__(&self) -> String {
//         format!("Monitor({})", self.name)
//     }
// }
// /// Options for creating a window. The ExperimentManager will try to find a
// /// video mode that satisfies the provided constraints. See documentation of the
// /// variants for more information.
// #[derive(Debug, Clone, PartialEq)]
// #[pyclass]
// pub enum WindowOptions {
//     Windowed {
//         /// The width and height of the window in pixels. Defaults to 800x600
//         /// (px).
//         resolution: Option<(u32, u32)>,
//     },
//     /// Match the given constraints exactly. You can set any of the constraints
//     /// to `None` to use the default value.
//     FullscreenExact {
//         /// The monitor to use. Defaults to the primary monitor.
//         monitor: Option<Monitor>,
//         /// The width and height of the window in pixels. Defaults to the width
//         /// of the first supported video mode of the selected monitor.
//         resolution: Option<(u32, u32)>,
//         /// The refresh rate to use in Hz. Defaults to the refresh rate of the
//         /// first supported video mode of the selected monitor.
//         refresh_rate: Option<f64>,
//     },
//     /// Select window configuration that satisfies the given constraints and has
//     /// the highest refresh rate.
//     FullscreenHighestRefreshRate {
//         monitor: Option<Monitor>,
//         resolution: Option<(u32, u32)>,
//     },
//     /// Select the highest resolution that satisfies the given constraints and
//     /// has the highest resolution.
//     FullscreenHighestResolution {
//         monitor: Option<Monitor>,
//         refresh_rate: Option<f64>,
//     },
// }

// impl WindowOptions {
//     /// Returns true if the window should be fullscreen.
//     pub fn fullscreen(&self) -> bool {
//         match self {
//             WindowOptions::Windowed { .. } => false,
//             WindowOptions::FullscreenExact { .. } => true,
//             WindowOptions::FullscreenHighestRefreshRate { .. } => true,
//             WindowOptions::FullscreenHighestResolution { .. } => true,
//         }
//     }

//     /// Returns the monitor that should be used for the window. If no monitor is
//     /// specified, returns None.
//     pub fn monitor(&self) -> Option<Monitor> {
//         match self {
//             WindowOptions::Windowed { .. } => None,
//             WindowOptions::FullscreenExact { monitor, .. } => monitor.clone(),
//             WindowOptions::FullscreenHighestRefreshRate { monitor, .. } => monitor.clone(),
//             WindowOptions::FullscreenHighestResolution { monitor, .. } => monitor.clone(),
//         }
//     }

//     /// Returns the resolution of the window. If no resolution is specified,
//     /// returns None.
//     pub fn resolution(&self) -> Option<(u32, u32)> {
//         match self {
//             WindowOptions::Windowed { resolution, .. } => *resolution,
//             WindowOptions::FullscreenExact { resolution, .. } => *resolution,
//             WindowOptions::FullscreenHighestRefreshRate { resolution, .. } => *resolution,
//             WindowOptions::FullscreenHighestResolution { .. } => None,
//         }
//     }

//     /// Returns the refresh rate of the window. If no refresh rate is specified,
//     /// returns None.
//     pub fn refresh_rate(&self) -> Option<f64> {
//         match self {
//             WindowOptions::Windowed { .. } => None,
//             WindowOptions::FullscreenExact { refresh_rate, .. } => *refresh_rate,
//             WindowOptions::FullscreenHighestRefreshRate { .. } => None,
//             WindowOptions::FullscreenHighestResolution { refresh_rate, .. } => *refresh_rate,
//         }
//     }
// }

// /// Custom event type for the event loop. This is used to communicate between
// /// the main thread and the render thread.
// #[derive(Dbg)]
// pub enum PsyEventLoopEvent {
//     PromptEvent(String, Sender<String>),
//     CreateNewWindowEvent(WindowOptions, Sender<WrappedWindow>),
//     NewWindowCreatedEvent(Window),
//     RunOnMainThread(#[dbg(placeholder = "...")] Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>),
// }

// /// The GPUState struct holds the state of the wgpu device and queue. It is used
// /// to create new windows.
// #[derive(Debug)]
// pub struct GPUState {
//     pub instance: wgpu::Instance,
//     pub adapter: wgpu::Adapter,
//     pub device: wgpu::Device,
//     pub queue: wgpu::Queue,
// }

// /// The MainLoop is the root element of the psydk library.
// #[derive(Debug)]
// #[pyclass(unsendable)]
// pub struct MainLoop {
//     /// The winit event loop
//     pub(crate) event_loop: Option<EventLoop<PsyEventLoopEvent>>,
//     /// Channel for sending a future to the render task. The future will be
//     /// executed on the render thread.
//     pub(crate) render_thread_channel_sender: Sender<RenderThreadChannelPayload>,
//     /// Channel for receiving functions that should be executed on the render
//     /// thread.
//     pub(crate) render_thread_channel_receiver: Receiver<RenderThreadChannelPayload>,
//     /// Vector of currently open windows
//     pub(crate) windows: Vec<WrappedWindow>,
//     /// The current GPU state
//     pub(crate) gpu_state: Arc<Mutex<GPUState>>,
// }

// /// The ExperimentManager is available to the user in the experiment function.
// #[derive(Debug)]
// #[pyclass(unsendable)]
// pub struct ExperimentManager {
//     event_loop_proxy: winit::event_loop::EventLoopProxy<PsyEventLoopEvent>,
//     available_monitors: Vec<Monitor>,
//     render_taks_sender: Sender<RenderThreadChannelPayload>,
// }

// impl ExperimentManager {
//     /// Show a prompt to the user. This function will block until the user has
//     /// entered a string and pressed enter.
//     pub fn prompt(&self, message: &str) -> String {
//         // dispatch a new UserEvent to the event loop
//         let (sender, receiver) = bounded(1);
//         let user_event = PsyEventLoopEvent::PromptEvent(message.to_string(), sender);

//         // send event
//         self.event_loop_proxy
//             .send_event(user_event)
//             .expect("Failed to send event to event loop. This is likely a bug, please report it.");

//         // wait for response
//         receiver
//             .recv_blocking()
//             .expect("Failed to receive response from event loop. This is likely a bug, please report it.")
//     }

//     /// Create a new window with the given options. This function will dispatch
//     /// a new UserEvent to the event loop and wait until the winit window
//     /// has been created. Then it will setup the wgpu device and surface and
//     /// return a new Window object.
//     pub fn create_window(&self, window_options: &WindowOptions) -> WrappedWindow {
//         // set up window by dispatching a new CreateNewWindowEvent to the event loop
//         let (sender, receiver) = bounded(1);
//         let user_event = PsyEventLoopEvent::CreateNewWindowEvent(window_options.clone(), sender);

//         // send event
//         self.event_loop_proxy
//             .send_event(user_event)
//             .expect("Failed to send event to event loop.");
//         log::debug!("Requested new window, waiting for response");

//         // wait for response
//         let window = receiver.recv_blocking().unwrap();
//         log::debug!("New window successfully created");

//         return window;
//     }

//     /// Create a default window. This is a convenience function that creates a
//     /// window with the default options.
//     pub fn create_default_window(&self, monitor: Option<u32>) -> WrappedWindow {
//         // select monitor 1 if available
//         // find all monitors available
//         let monitors = self.get_available_monitors();
//         // get the second monitor if available, otherwise use the first one
//         let monitor = monitors
//             .get(monitor.unwrap_or(0) as usize)
//             .unwrap_or(monitors.first().expect("No monitor found - this should not happen"));

//         log::debug!("Creating default window on monitor {:?}", monitor);
//         self.create_window(&WindowOptions::FullscreenHighestResolution {
//             monitor: Some(monitor.clone()),
//             refresh_rate: None,
//         })
//     }

//     /// Retrive available monitors.
//     /// TODO: update available monitors when a monitor is connected or disconnected
//     pub fn get_available_monitors(&self) -> Vec<Monitor> {
//         self.available_monitors.clone()
//     }
// }

// #[pymethods]
// impl ExperimentManager {
//     #[pyo3(name = "create_default_window")]
//     #[pyo3(signature = (monitor = None))]
//     fn py_create_default_window(&self, monitor: Option<u32>) -> WrappedWindow {
//         self.create_default_window(monitor)
//     }

//     #[pyo3(name = "get_available_monitors")]
//     fn py_get_available_monitors(&self) -> Vec<Monitor> {
//         self.get_available_monitors()
//     }
// }

// impl MainLoop {
//     pub async fn new() -> Self {
//         // create channel for sending tasks to the render thread
//         let (render_task_sender, render_task_receiver) = bounded(100);
//         let event_loop = EventLoopBuilder::<PsyEventLoopEvent>::with_user_event()
//             .build()
//             .expect("Failed to create event loop. This is likely a bug, please report it.");

//         // this is where we would chose a specific backend
//         let backend = wgpu::Backends::all();

//         #[cfg(target_os = "windows")]
//         let backend = wgpu::Backends::VULKAN;

//         let instance_desc = wgpu::InstanceDescriptor {
//             backends: backend,
//             // use defaults for the rest
//             ..Default::default()
//         };

//         let instance = wgpu::Instance::new(instance_desc);

//         // request an adapter
//         let adapter = instance
//             .request_adapter(&wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::HighPerformance,
//                 force_fallback_adapter: false,
//                 compatible_surface: None, // idealy we would use the surface here, but we don't have it yet
//             })
//             .await
//             .expect("Failed to find an suitable graphics adapter. This is likely a bug, please report it.");

//         log::debug!("Selected graphics adapter: {:?}", adapter.get_info());

//         let mut limits = wgpu::Limits::downlevel_defaults();
//         limits.max_storage_buffers_per_shader_stage = 16;

//         // Create the logical device and command queue
//         let (device, queue) = adapter
//             .request_device(
//                 &wgpu::DeviceDescriptor {
//                     label: None,
//                     required_features: wgpu::Features::empty(),
//                     // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
//                     required_limits: limits.using_resolution(adapter.limits()),
//                     memory_hints: MemoryHints::Performance,
//                 },
//                 None,
//             )
//             .await
//             .expect("Failed to create device. This is likely a bug, please report it.");

//         Self {
//             event_loop: Some(event_loop),
//             render_thread_channel_sender: render_task_sender,
//             render_thread_channel_receiver: render_task_receiver,
//             windows: vec![],
//             gpu_state: Arc::new(Mutex::new(GPUState {
//                 instance,
//                 adapter,
//                 device,
//                 queue,
//             })),
//         }
//     }

//     /// Create a new window with the given options.
//     pub fn create_window(
//         &self,
//         window_options: &WindowOptions,
//         event_loop_target: &EventLoopWindowTarget<PsyEventLoopEvent>,
//     ) -> WrappedWindow {
//         let fullscreen_mode = if window_options.fullscreen() {
//             // get monitor
//             let monitor_handle = if let Some(monitor) = window_options.monitor() {
//                 monitor.handle
//             } else {
//                 event_loop_target
//                     .primary_monitor()
//                     .expect("No primary monitor found. If a screen is connected, this is a bug, please report it.")
//             };

//             log::debug!(
//                 "Video modes: {:?}",
//                 monitor_handle.video_modes().collect::<Vec<VideoMode>>()
//             );

//             // filter by resolution if specified
//             let video_modes: Vec<VideoMode> = if let Some(resolution) = window_options.resolution() {
//                 monitor_handle
//                     .video_modes()
//                     .filter(|video_mode| {
//                         video_mode.size().width == resolution.0 && video_mode.size().height == resolution.1
//                     })
//                     .collect()
//             } else {
//                 monitor_handle.video_modes().collect()
//             };

//             // filter by refresh rate if specified
//             let mut video_modes: Vec<VideoMode> = if let Some(refresh_rate) = window_options.refresh_rate() {
//                 video_modes
//                     .into_iter()
//                     .filter(|video_mode| video_mode.refresh_rate_millihertz() as f64 / 1000.0 == refresh_rate)
//                     .collect()
//             } else {
//                 video_modes
//             };

//             // sort by refresh rate
//             video_modes.sort_by(|a, b| a.refresh_rate_millihertz().cmp(&b.refresh_rate_millihertz()));

//             // sort by resolution (width*height)
//             video_modes.sort_by(|a, b| (a.size().width * a.size().height).cmp(&(b.size().width * b.size().height)));

//             log::debug!("Video modes: {:?}", video_modes);

//             // match the type of window_options
//             let video_mode = match window_options {
//                 WindowOptions::FullscreenExact { .. } => video_modes
//                     .first()
//                     .expect("No suitable video modes found, please check your window options.")
//                     .clone(),
//                 WindowOptions::FullscreenHighestRefreshRate { .. } => {
//                     // filter by refresh rate
//                     video_modes
//                         .clone()
//                         .into_iter()
//                         .filter(|video_mode| {
//                             video_mode.refresh_rate_millihertz()
//                                 == video_modes.last().unwrap().refresh_rate_millihertz()
//                         })
//                         .collect::<Vec<VideoMode>>()
//                         .first()
//                         .expect("No suitable video modes found, please check your window options.")
//                         .clone()
//                 }
//                 WindowOptions::FullscreenHighestResolution { .. } => {
//                     // filter by resolution
//                     video_modes
//                         .clone()
//                         .into_iter()
//                         .filter(|video_mode| {
//                             video_mode.size().width * video_mode.size().height
//                                 == video_modes.last().unwrap().size().width * video_modes.last().unwrap().size().height
//                         })
//                         .collect::<Vec<VideoMode>>()
//                         .first()
//                         .expect("No suitable video modes found, please check your window options.")
//                         .clone()
//                 }
//                 _ => panic!("Invalid window options"),
//             };

//             // create window
//             Some(winit::window::Fullscreen::Borderless(Some(monitor_handle)))
//         } else {
//             // we just create a window on the specified monitor

//             None
//         };

//         let window_attributes = Window::default_attributes()
//             .with_title("Winit window")
//             .with_transparent(false);

//         let winit_window = event_loop.create_window(window_attributes).unwrap();

//         // make sure cursor is visible (for normlisation across platforms)
//         winit_window.set_cursor_visible(true);

//         winit_window.focus_window();

//         // log::debug!("Window created: {:?}", winit_window);

//         let winit_window = Arc::new(winit_window);

//         let gpu_state = self.gpu_state.lock().unwrap();

//         let instance = &gpu_state.instance;
//         let adapter = &gpu_state.adapter;
//         let device = &gpu_state.device;

//         log::debug!("Creating wgup surface...");

//         let surface = instance
//             .create_surface(winit_window.clone())
//             .expect("Failed to create surface. This is likely a bug, please report it.");

//         // print supported swapchain formats
//         let swapchain_formats = surface.get_capabilities(&adapter).formats;
//         log::debug!("Supported swapchain formats: {:?}", swapchain_formats);

//         let size = winit_window.inner_size();

//         let _swapchain_formats = adapter.get_texture_format_features(TextureFormat::Bgra8Unorm);

//         let swapchain_capabilities = surface.get_capabilities(&adapter);
//         let swapchain_format = TextureFormat::Bgra8Unorm;
//         let swapchain_view_format = vec![TextureFormat::Bgra8Unorm];

//         let config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format: swapchain_format,
//             width: size.width,
//             height: size.height,
//             present_mode: wgpu::PresentMode::Fifo,
//             alpha_mode: swapchain_capabilities.alpha_modes[0],
//             view_formats: swapchain_view_format,
//             desired_maximum_frame_latency: 2,
//         };

//         log::debug!("Surface configuration: {:?}", config);

//         surface.configure(&device, &config);

//         // set fullscreen mode
//         winit_window.set_fullscreen(fullscreen_mode);

//         // create the renderer
//         let mut renderer = renderer::DynamicRenderer::new(
//             renderer::Backend::Skia,
//             &device,
//             swapchain_format,
//             size.width,
//             size.height,
//         );

//         let wgpu_renderer = pollster::block_on(wgpu_renderer::WgpuRenderer::new(winit_window));

//         let winit_id = winit_window.id();

//         // create a pwindow
//         let window_state = InternalWindowState {
//             winit_window: winit_window.clone(),
//             surface,
//             config,
//             renderer,
//             wgpu_renderer,
//         };

//         // create channel for physical input
//         let (mut event_broadcast_sender, physical_input_receiver) = async_broadcast::broadcast(10_000);
//         event_broadcast_sender.set_overflow(true);
//         // deactivate the receiver
//         let event_broadcast_receiver = physical_input_receiver.deactivate();

//         // create physical properties
//         let physical_properties = WindowPhysicalProperties {
//             width: size.width,
//             height: size.height,
//             width_m: 0.3,
//             pixel_aspect_ratio: 1.0,
//             viewing_distance: 57.0,
//         };

//         // create handle
//         let window = WrappedWindow::new(Window {
//             winit_id: winit_id,
//             state: window_state,
//             gpu_state: self.gpu_state.clone(),
//             mouse_position: None,
//             mouse_cursor_visible: true,
//             event_broadcast_receiver,
//             event_broadcast_sender,
//             stimuli: Arc::new(Mutex::new(vec![])),
//             event_handlers: Arc::new(Mutex::new(HashMap::new())),
//             options: Arc::new(Mutex::new(options::GlobalOptions::default())),
//             physical_properties,
//         });

//         let win_clone = window.clone();
//         // add a default event handler for mouse move events, which updates the mouse
//         // position
//         window.add_event_handler(EventKind::CursorMoved, move |event| {
//             if let Some(pos) = event.position() {
//                 win_clone.inner().mouse_position = Some(pos.clone());
//             };
//             false
//         });

//         return window;
//     }

//     // pub fn get_available_monitors(&mut self) -> Vec<Monitor> {
//     //     let mut monitors = vec![];
//     //     let event_loop = self.event_loop.as_ref().unwrap();
//     //     for (i, handle) in event_loop.available_monitors().enumerate() {
//     //         monitors.push(Monitor {
//     //             name: handle.name().unwrap_or(format!("Unnamed monitor {}", i)),
//     //             resolution: (handle.size().width, handle.size().height),
//     //             handle: handle,
//     //         });
//     //     }
//     //     monitors
//     // }

//     /// Starts the experiment. This will block until the experiment.
//     ///
//     /// # Arguments
//     ///
//     /// * `experiment_fn` - The function that is your experiment. This function
//     ///   will be called with a `Window` object that you can use to create
//     ///   stimuli and submit frames to the window.
//     pub fn run_experiment<F>(&mut self, experiment_fn: F) -> ()
//     where
//         F: FnOnce(ExperimentManager) -> Result<(), errors::psydkError> + 'static + Send,
//     {
//         let event_loop = self.event_loop.take().unwrap();

//         #[cfg(not(target_arch = "wasm32"))]
//         {
//             smol::block_on(self.run_event_loop(event_loop, experiment_fn));
//         }
//     }

//     async fn run_event_loop<F>(&mut self, event_loop: EventLoop<PsyEventLoopEvent>, experiment_fn: F)
//     where
//         F: FnOnce(ExperimentManager) -> Result<(), errors::psydkError> + 'static + Send,
//     {
//         log::debug!("Main task is running on thread {:?}", std::thread::current().id());

//         let available_monitors = event_loop
//             .available_monitors()
//             .map(|monitor| Monitor {
//                 name: monitor.name().unwrap_or("Unnamed monitor".to_string()),
//                 resolution: (monitor.size().width, monitor.size().height),
//                 handle: monitor,
//             })
//             .collect();

//         let wm = ExperimentManager {
//             event_loop_proxy: event_loop.create_proxy(),
//             render_taks_sender: self.render_thread_channel_sender.clone(),
//             available_monitors: available_monitors,
//         };

//         // // start renderer
//         // {
//         //     let win_handle = window.clone();
//         //     #[cfg(target_arch = "wasm32")]
//         //     spawn_async_task(render_task(window));
//         //     #[cfg(not(target_arch = "wasm32"))]
//         //     thread::spawn(move || {
//         //         smol::block_on(render_task(win_handle));
//         //     });
//         // }

//         // // start renderer2 on WASM, we run this async in the browsers event loop
//         // // on native, we spawn a new thread (but we might need to change that)
//         // {
//         //     let win_handle = window.clone();
//         //     #[cfg(target_arch = "wasm32")]
//         //     spawn_async_task(render_task2(window));
//         //     #[cfg(not(target_arch = "wasm32"))]
//         //     let rpr = self.render_task_receiver.clone();
//         //     thread::spawn(move || {
//         //         smol::block_on(render_task2(rpr));
//         //     });
//         // }

//         // start experiment
//         thread::spawn(move || {
//             let res = experiment_fn(wm);
//             // panic if the experiment function returns an error
//             if let Err(e) = res {
//                 // throw error
//                 log::error!("Experiment failed with {:?}: {:}", e, e);
//                 // quit program
//                 std::process::exit(1);
//             }
//         });

//         // set event loop to poll
//         event_loop.set_control_flow(ControlFlow::Poll);

//         let _ = event_loop.run(move |event: WinitEvent<PsyEventLoopEvent>, win_target| {
//             match event {
//                 WinitEvent::UserEvent(event) => {
//                     match event {
//                         PsyEventLoopEvent::CreateNewWindowEvent(window_options, sender) => {
//                             log::debug!("Event loop received CreateNewWindowEvent - creating new window");

//                             let window = self.create_window(&window_options, win_target);

//                             // push window to list of windows
//                             self.windows.push(window.clone());

//                             sender
//                                 .send_blocking(window)
//                                 .expect("Failed to send window to sender. This is likely a bug, please report it.");
//                         }
//                         PsyEventLoopEvent::NewWindowCreatedEvent(_window) => {
//                             log::debug!("Event loop received NewWindowCreatedEvent");
//                             // add window to list of windows
//                             //self.windows.push(window);
//                         }
//                         PsyEventLoopEvent::RunOnMainThread(task) => {
//                             log::debug!("Running task on main thread");
//                             let _ = block_on(task());
//                         }
//                         _ => {
//                             log::debug!("Event loop received unhandled event");
//                         }
//                     }
//                 }
//                 WinitEvent::WindowEvent {
//                     window_id: id,
//                     event: WindowEvent::Resized(new_size),
//                 } => {
//                     if let Some(mut window) = self.get_window_by_id(id).clone() {
//                         // update window size

//                         let mut win_inner = window.inner();
//                         win_inner.physical_properties.width = new_size.width;
//                         win_inner.physical_properties.height = new_size.height;

//                         let mut window_state = &mut win_inner.state;
//                         let gpu_state = self.gpu_state.lock().unwrap();

//                         window_state.config.width = new_size.width.max(1);
//                         window_state.config.height = new_size.height.max(1);

//                         window_state.surface.configure(&gpu_state.device, &window_state.config);

//                         // on macos, the window size is not updated automatically
//                         window_state.winit_window.request_redraw();

//                         // resize renderer
//                         window_state
//                             .renderer
//                             .resize(&gpu_state.device, new_size.width, new_size.height);
//                     }
//                 }

//                 // handle window events
//                 WinitEvent::WindowEvent { window_id: id, event } => {
//                     if let Some(window) = self.get_window_by_id(id) {
//                         //println!("Window found");
//                         if let Some(input) = Event::try_from_winit(event.clone(), &window).ok() {
//                             // if escape key was pressed, close window
//                             if input.key_pressed("\u{1b}") {
//                                 win_target.exit();
//                             }

//                             // check if the window mutex is locked (for debugging)
//                             //window.0.try_lock().expect("Window mutex is locked");

//                             // broadcast event to window
//                             window.inner().event_broadcast_sender.try_broadcast(input.clone());

//                             // dispatch_event to window
//                             // note: this should be done in a separate thread using the winndow's event_broadcast channel
//                             window.dispatch_event(input);
//                         }
//                         //println!("Window event handled");
//                     }
//                 }
//                 // handle close event
//                 _ => {}
//             }
//         });
//     }

//     pub fn get_window_by_id(&self, id: winit::window::WindowId) -> Option<WrappedWindow> {
//         for window in &self.windows {
//             if window.winit_id() == id {
//                 return Some(window.clone());
//             }
//         }
//         None
//     }
// }

// #[pymethods]
// impl MainLoop {
//     #[new]
//     fn py_new() -> Self {
//         smol::block_on(MainLoop::new())
//     }

//     /// Runs your experiment function. This function will block the current thread
//     /// until the experiment function returns.
//     /// returns.
//     ///
//     /// Parameters
//     /// ----------
//     /// experiment_fn : callable
//     ///    The function that runs your experiment. This function should take a single argument, an instance of `ExperimentManager`, and should not return nothing.
//     #[pyo3(name = "run_experiment", signature = (experiment_fn, *args, **kwargs))]
//     fn py_run_experiment(
//         &mut self,
//         py: Python,
//         experiment_fn: Py<PyAny>,
//         args: Py<PyTuple>,
//         kwargs: Option<Py<PyDict>>,
//     ) {
//         let rust_experiment_fn = move |em: ExperimentManager| -> Result<(), errors::psydkError> {
//             Python::with_gil(|py| -> _ {
//                 // bind kwargs
//                 let kwargs = if let Some(kwargs) = kwargs {
//                     kwargs.into_bound(py)
//                 } else {
//                     PyDict::new_bound(py)
//                 };

//                 // TODO: There must be a better way to do this
//                 let args = args.bind(py);
//                 let args_as_seq = args.to_list();
//                 let args_as_seq = args_as_seq.as_sequence();
//                 let em = em.into_py(py);
//                 let em_as_seq = PyList::new_bound(py, vec![em]);
//                 let em_as_seq = em_as_seq.as_sequence();

//                 let args = em_as_seq.concat(args_as_seq).unwrap();
//                 let args = args.to_tuple().unwrap();

//                 experiment_fn.call_bound(py, args, Some(&kwargs))
//             })
//             .unwrap();
//             Ok(())
//         };

//         let mut self_wrapper = send_wrapper::SendWrapper::new(self);

//         py.allow_threads(move || self_wrapper.run_experiment(rust_experiment_fn));
//     }
// }

// // /// This is the second render task. It is used to execute tasks on the render
// // thread when running on WASM. async fn render_task2(
// //     render_task_receiver: Receiver<
// //         Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
// //     >,
// // ) {
// //     log::debug!(
// //         "Render task 2 running on thread {:?}",
// //         std::thread::current().id()
// //     );
// //     // loop forever
// //     loop {
// //         // wait until task is available
// //         let task = render_task_receiver.recv().await.unwrap();
// //         // await the task (the task itself will handle the backchannel)
// //         task().await;
// //     }
// // }
