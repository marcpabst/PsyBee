
use async_channel::{bounded, Receiver, Sender};
use async_lock::{Mutex, RwLock};

use atomic_float::AtomicF64;
use futures_lite::future::block_on;
use futures_lite::Future;

#[cfg(target_os = "macos")]
use objc2::rc::Id;
#[cfg(target_os = "macos")]
use objc2_foundation::{CGPoint, CGSize, NSRect, ns_string, MainThreadMarker};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSAlertStyle, NSAlert, NSTextField};

use winit::monitor::VideoMode;

use crate::visual::color::ColorFormat;

use crate::input::Event;

use async_executor::Executor;



use wgpu::TextureFormat;

use std::fmt::Formatter;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;



use winit::event::{Event as WinitEvent, WindowEvent};
use winit::event_loop::{
    ControlFlow, EventLoopBuilder, EventLoopWindowTarget,
};

// this is behind a feature flag because it is not yet stable
#[cfg(feature = "gst")]
pub mod camera;

pub mod errors;
pub mod input;

pub mod utils;
pub mod visual;
use winit::event_loop::EventLoop;

// re-export wgpu
pub use wgpu;


// the prelude
pub mod prelude {
    pub use crate::errors::PsychophysicsError;
    pub use crate::input::Key;
    pub use crate::input::EventReceiver;
    pub use crate::loop_frames;
    #[cfg(feature = "serial")]
    pub use crate::serial::SerialPort;
    pub use crate::utils::sleep_secs;
    pub use crate::utils::BIDSEventLogger;
    pub use crate::visual::color;
    pub use crate::visual::geometry::{Rectangle, Size};
    pub use crate::visual::stimuli::PatternStimulus;
    // pub use crate::visual::stimuli::ImageStimulus;
    pub use crate::visual::stimuli::ColorStimulus;
    // pub use crate::visual::{stimuli::TextStimulus, Window};
    pub use crate::visual::geometry::Transformation2D;
    pub use crate::visual::geometry::Circle;
}

#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;

use crate::visual::window::{
    render_task, Frame, Window, WindowState,
};


#[cfg(target_arch = "wasm32")]
pub fn web_window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

#[cfg(target_arch = "wasm32")]
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_async_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

// get the global executor
fn get_executor() -> &'static Executor<'static> {
    static EXECUTOR: Executor<'static> = Executor::new();
    &EXECUTOR
}

#[cfg(not(target_arch = "wasm32"))]
pub trait FutureReturnTrait:
    Future<Output = ()> + 'static + Send
{
}
#[cfg(not(target_arch = "wasm32"))]
impl<F> FutureReturnTrait for F where
    F: Future<Output = ()> + 'static + Send
{
}
#[cfg(target_arch = "wasm32")]
pub trait FutureReturnTrait: Future<Output = ()> + 'static {}
#[cfg(target_arch = "wasm32")]
impl<F> FutureReturnTrait for F where F: Future<Output = ()> + 'static {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Monitor {
    pub name: String,
    handle: winit::monitor::MonitorHandle,
}

/// Options for creating a window. The ExperimentManager will try to find a video mode that satisfies the provided constraints.
/// See documentation of the variants for more information.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowOptions {
    Windowed {
        /// The width and height of the window in pixels. Defaults to 800x600 (px).
        resolution: Option<(u32, u32)>,
    },
    /// Match the given constraints exactly. You can set any of the constraints to `None` to use the default value.
    FullscreenExact {
        /// The monitor to use. Defaults to the primary monitor.
        monitor: Option<Monitor>,
        /// The width and height of the window in pixels. Defaults to the width of the first supported video mode of the selected monitor.
        resolution: Option<(u32, u32)>,
        /// The refresh rate to use in Hz. Defaults to the refresh rate of the first supported video mode of the selected monitor.
        refresh_rate: Option<f64>,
    },
    /// Select window configuration that satisfies the given constraints and has the highest refresh rate.
    FullscreenHighestRefreshRate {
        monitor: Option<Monitor>,
        resolution: Option<(u32, u32)>,
    },
    /// Select the highest resolution that satisfies the given constraints and has the highest resolution.
    FullscreenHighestResolution {
        monitor: Option<Monitor>,
        refresh_rate: Option<f64>,
    },
}

impl WindowOptions {
    pub fn fullscreen(&self) -> bool {
        match self {
            WindowOptions::Windowed { .. } => false,
            WindowOptions::FullscreenExact { .. } => true,
            WindowOptions::FullscreenHighestRefreshRate {
                ..
            } => true,
            WindowOptions::FullscreenHighestResolution { .. } => true,
        }
    }

    pub fn monitor(&self) -> Option<Monitor> {
        match self {
            WindowOptions::Windowed { .. } => None,
            WindowOptions::FullscreenExact { monitor, .. } => {
                monitor.clone()
            }
            WindowOptions::FullscreenHighestRefreshRate {
                monitor,
                ..
            } => monitor.clone(),
            WindowOptions::FullscreenHighestResolution {
                monitor,
                ..
            } => monitor.clone(),
        }
    }

    pub fn resolution(&self) -> Option<(u32, u32)> {
        match self {
            WindowOptions::Windowed { resolution, .. } => *resolution,
            WindowOptions::FullscreenExact { resolution, .. } => {
                *resolution
            }
            WindowOptions::FullscreenHighestRefreshRate {
                resolution,
                ..
            } => *resolution,
            WindowOptions::FullscreenHighestResolution { .. } => None,
        }
    }

    pub fn refresh_rate(&self) -> Option<f64> {
        match self {
            WindowOptions::Windowed { .. } => None,
            WindowOptions::FullscreenExact {
                refresh_rate, ..
            } => *refresh_rate,
            WindowOptions::FullscreenHighestRefreshRate {
                ..
            } => None,
            WindowOptions::FullscreenHighestResolution {
                refresh_rate,
                ..
            } => *refresh_rate,
        }
    }
}

pub enum PsychophysicsEventLoopEvent {
    PromptEvent(String, Sender<String>),
    CreateNewWindowEvent(WindowOptions, Sender<Window>),
    NewWindowCreatedEvent(Window),
    RunOnMainThread(Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,>),
}

impl std::fmt::Debug for PsychophysicsEventLoopEvent
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PsychophysicsEventLoopEvent::CreateNewWindowEvent(_, _) => {
                write!(f, "CreateNewWindowEvent")
            },
            PsychophysicsEventLoopEvent::NewWindowCreatedEvent(_) => {
                write!(f, "NewWindowCreatedEvent")
            },
            PsychophysicsEventLoopEvent::RunOnMainThread(_) => {
                write!(f, "RunOnMainThread")
            },
            PsychophysicsEventLoopEvent::PromptEvent(_, _) => {
                write!(f, "PromptEvent")
            },
        }
    }
}

/// The GPUState struct holds the state of the wgpu device and queue. It is used to create new windows.
#[derive(Debug)]
pub struct GPUState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// The ExperimentManager is the root element of the psychophysics library.
/// It is responsible for creating and managing window(s) and for running
/// one or more experiment(s). The ExperimentManager is a singleton and
/// can be created using the `new` method. No more than one ExperimentManager
/// can exist at any given time.
#[derive(Debug)]
pub struct ExperimentManager {
    event_loop: Option<EventLoop<PsychophysicsEventLoopEvent>>,
    event_loop_proxy: winit::event_loop::EventLoopProxy<
        PsychophysicsEventLoopEvent,
    >,
    /// Channel for sending a future to the render task. The future will be executed on the render thread.
    pub render_task_sender: Sender<
        Box<
            dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    >,
    /// Channel for receiving functions that should be executed on the render thread.
    pub render_task_receiver: Receiver<
        Box<
            dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    >,

    /// active Windows
    pub windows: Vec<Window>,

    // wgpu stuff
    pub gpu_state: Arc<RwLock<GPUState>>,
}

/// The WindowManager is available as an argument to the experiment function. It can be used to create new windows.
#[derive(Debug)]
pub struct WindowManager {
    event_loop_proxy: winit::event_loop::EventLoopProxy<
        PsychophysicsEventLoopEvent,
    >,
    available_monitors: Vec<Monitor>,
    render_taks_sender: Sender<Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>>,
}

impl WindowManager {

    /// Show a prompt to the user. This function will block until the user has entered a string and pressed enter.
    pub fn prompt(&self, message: &str) -> String {
        // dispatch a new UserEvent to the event loop
        let (sender, receiver) = bounded(1);
        let user_event = PsychophysicsEventLoopEvent::PromptEvent(
            message.to_string(),
            sender,
        );

        // send event
        self.event_loop_proxy.send_event(user_event).expect("Failed to send event to event loop. This is likely a bug, please report it.");

        // wait for response
        let response = receiver.recv_blocking().expect("Failed to receive response from event loop. This is likely a bug, please report it.");
        return response;
        //return receiver.recv_blocking().unwrap();
    }

    /// Create a new window with the given options. This function will dispatch a new UserEvent to the event loop
    /// and wait until the winit window has been created. Then it will setup the wgpu device and surface and return
    /// a new Window object.
    pub fn create_window(
        &self,
        window_options: &WindowOptions,
    ) -> Window {
       // dispatch a new UserEvent to the event loop
                // set up window by dispatching a new UserEvent to the event loop
                let (sender, receiver) = bounded(1);
                let user_event = PsychophysicsEventLoopEvent::CreateNewWindowEvent(
                    window_options.clone(),
                    sender,
                );
        
                // send event
                self.event_loop_proxy.send_event(user_event).expect("Failed to send event to event loop. This is likely a bug, please report it.");
                
                
                log::debug!("Requested new window, waiting for response");

                // wait for response
                let window = receiver.recv_blocking().unwrap();
                log::debug!("New window successfully created");

                return window;
    }

    /// Create a default window. This is a convenience function that creates a window with the default options.
    pub fn create_default_window(&self) -> Window {
        // select monitor 1 if available
            // find all monitors available
        let monitors = self.get_available_monitors();
        // get the second monitor if available, otherwise use the first one
        let monitor = monitors.get(1).unwrap_or(
            monitors
                .first()
                .expect("No monitor found - this should not happen"),
        );

        log::debug!("Creating default window on monitor {:?}", monitor);
        self.create_window(&WindowOptions::FullscreenHighestResolution {  monitor: Some(monitor.clone()), refresh_rate: None })
    }

    /// Retrive available monitors. This reflects the state of the monitors at the time of the creation of the WindowManager.
    /// If a monitor is connected or disconnected after the WindowManager has been created, this will not be reflected here!
    pub fn get_available_monitors(&self) -> Vec<Monitor> {
        self.available_monitors.clone()
    }
}

impl ExperimentManager {
    /// Create a new ExperimentManager.
    pub async fn new() -> Self {
        // create channel for sending tasks to the render thread
        let (render_task_sender, render_task_receiver) = bounded(100);
        let event_loop = EventLoopBuilder::<PsychophysicsEventLoopEvent>::with_user_event().build().expect("Failed to create event loop. This is likely a bug, please report it.");
        let event_loop_proxy = event_loop.create_proxy();

        // let instance_desc = wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN | wgpu::Backends::METAL,
        //     // use defaults for the rest
        //     ..Default::default()
        // };

        // chose any backend except on windows, where we prefer DX12
        // #[cfg(target_os = "windows")]
        // let backend = wgpu::Backends::DX12;
        // #[cfg(not(target_os = "windows"))]
        let backend = wgpu::Backends::all();

        let instance_desc = wgpu::InstanceDescriptor {
            backends: backend,
            // use defaults for the rest
            ..Default::default()
        };

        let instance = wgpu::Instance::new(instance_desc);


        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: None, // idealy we would use the surface here, but we don't have it yet
            })
            .await
            .expect("Failed to find an appropiate graphics adapter. This is likely a bug, please report it.");

        log::debug!("Adapter: {:?}", adapter.get_info());

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::default()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device. This is likely a bug, please report it.");
        
        Self {
            event_loop: Some(event_loop),
            event_loop_proxy,
            render_task_sender,
            render_task_receiver,
            windows: vec![],
            gpu_state: Arc::new(RwLock::new(GPUState {
                instance,
                adapter,
                device,
                queue,
            })),
        }
    }

    /// Create a new window with the given options.
    pub fn create_window(
        &self,
        window_options: &WindowOptions,
        event_loop_target: &EventLoopWindowTarget<PsychophysicsEventLoopEvent>,
    ) -> Window {
        // set up window
        let winit_window: winit::window::Window;

        if window_options.fullscreen() {
            // get monitor
            let monitor_handle = if let Some(monitor) =
                window_options.monitor()
            {
                monitor.handle
            } else {
                event_loop_target.primary_monitor().expect("No primary monitor found. If a screen is connected, this is a bug, please report it.")
            };

            log::debug!("Video modes: {:?}", monitor_handle.video_modes().collect::<Vec<VideoMode>>());

            // filter by resolution if specified
            let video_modes: Vec<VideoMode> =
                if let Some(resolution) = window_options.resolution()
                {
                    monitor_handle
                        .video_modes()
                        .filter(|video_mode| {
                            video_mode.size().width as u32
                                == resolution.0
                                && video_mode.size().height as u32
                                    == resolution.1
                        })
                        .collect()
                } else {
                    monitor_handle.video_modes().collect()
                };


            // filter by refresh rate if specified
            let mut video_modes: Vec<VideoMode> =
                if let Some(refresh_rate) =
                    window_options.refresh_rate()
                {
                    video_modes
                        .into_iter()
                        .filter(|video_mode| {
                            video_mode.refresh_rate_millihertz()
                                as f64
                                / 1000.0
                                == refresh_rate
                        })
                        .collect()
                } else {
                    video_modes
                };


            // sort by refresh rate
            video_modes.sort_by(|a, b| {
                a.refresh_rate_millihertz()
                    .cmp(&b.refresh_rate_millihertz())
            });

            // sort by resolution (width*height)
            video_modes.sort_by(|a, b| {
                (a.size().width * a.size().height)
                    .cmp(&(b.size().width * b.size().height))
            });

            log::debug!("Video modes: {:?}", video_modes);

            // match the type of window_options
            let video_mode = match window_options {
              WindowOptions::FullscreenExact { .. } => {
                  video_modes.first().expect("No suitable video modes found, please check your window options.").clone()
              },
              WindowOptions::FullscreenHighestRefreshRate { .. } => {
                  // filter by refresh rate
                video_modes.clone().into_iter().filter(|video_mode| {
                      video_mode.refresh_rate_millihertz() == video_modes.last().unwrap().refresh_rate_millihertz()
                  }).collect::<Vec<VideoMode>>().first().expect("No suitable video modes found, please check your window options.").clone()
              },
              WindowOptions::FullscreenHighestResolution { .. } => {
                  // filter by resolution
                  video_modes.clone().into_iter().filter(|video_mode| {
                      video_mode.size().width * video_mode.size().height == video_modes.last().unwrap().size().width * video_modes.last().unwrap().size().height
                  }).collect::<Vec<VideoMode>>().first().expect("No suitable video modes found, please check your window options.").clone()
              },
              _ => panic!("Invalid window options")
          };

            // create window
            winit_window = winit::window::WindowBuilder::new()
                // make exclusive fullscreen
                .with_fullscreen(Some(
                    winit::window::Fullscreen::Exclusive(video_mode),
                ))
                .with_title("Experiment".to_string())
                .build(&event_loop_target)
                .unwrap();
        } else {
            // we just create a window on the specified monitor

            winit_window = winit::window::WindowBuilder::new()
                // make exclusive fullscreen
                .with_fullscreen(None)
                .with_title("Experiment".to_string())
                .build(&event_loop_target)
                .unwrap();
        }

        // hide cursor
        // winit_window.set_cursor_visible(false);
        winit_window.focus_window();

        log::debug!("Window created: {:?}", winit_window);

        let winit_window = Arc::new(winit_window);


        let gpu_state = self.gpu_state.read_blocking();
        let instance = &gpu_state.instance;
        let adapter = &gpu_state.adapter;
        let device = &gpu_state.device;

        log::debug!("Creating wgup surface...");

        let surface =  
            instance.create_surface(winit_window.clone()).expect("Failed to create surface. This is likely a bug, please report it.");
     
        // // get HAL using callback (but only on macos)
        // let hal_surface =  unsafe { surface.as_hal::<wgpu::hal::api::Dx12, _, _>(
        //     |surface| {
        //         let surface = surface.unwrap();
        //         println!("Surface: {:?}", surface.present_with_transaction);
        //     }
        // )
        // };
        

        // print supported swapchain formats
        let swapchain_formats = surface.get_capabilities(&adapter).formats;
        log::debug!("Supported swapchain formats: {:?}", swapchain_formats);

        let size = winit_window.inner_size();
 
        let _swapchain_formats = adapter.get_texture_format_features(TextureFormat::Bgra8Unorm);

 
         let swapchain_capabilities = surface.get_capabilities(&adapter);
         let swapchain_format = TextureFormat::Bgra8Unorm;
         let swapchain_view_format = vec![
             TextureFormat::Bgra8Unorm,
         ];
     
         let config = wgpu::SurfaceConfiguration {
             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
             format: swapchain_format,
             width: size.width,
             height: size.height,
             present_mode: wgpu::PresentMode::Fifo,
             alpha_mode: swapchain_capabilities.alpha_modes[0],
             view_formats: swapchain_view_format,
             desired_maximum_frame_latency: 1,
         };

         log::debug!("Surface configuration: {:?}", config);
     
         surface.configure(&device, &config);
     
         // create channel for frame submission
         let (frame_sender, frame_receiver): (
             Sender<Arc<Mutex<Frame>>>,
             Receiver<Arc<Mutex<Frame>>>,
         ) = bounded(1);
     
         let (frame_ok_sender, frame_ok_receiver): (
             Sender<bool>,
             Receiver<bool>,
         ) = bounded(1);
     
         // create a pwindow
         let window_state = WindowState {
             window: winit_window.clone(),
             surface,
             config,
         };
 
         // create channel for physical input
         let (mut physical_input_sender, physical_input_receiver) = async_broadcast::broadcast(10000);
         physical_input_sender.set_overflow(true);
         // deactivate the receiver
         let physical_input_receiver = physical_input_receiver.deactivate();
     
         // create handle
         let window = Window {
             state: Arc::new(RwLock::new(window_state)),
             gpu_state: self.gpu_state.clone(),
             event_receiver: physical_input_receiver,
             physical_input_sender,
             frame_sender,
             frame_receiver,
             frame_ok_sender,
             frame_ok_receiver,
             physical_width: Arc::new(AtomicF64::new(300.0)),
             viewing_distance: Arc::new(AtomicF64::new(57.0)),
             color_format: ColorFormat::SRGBA8,
             width_px: Arc::new(AtomicU32::new(300)),
             height_px: Arc::new(AtomicU32::new(300)),
             render_task_sender: self.render_task_sender.clone(),
             //event_handlers: vec![],
         };
     
 
         return window;
}

    /// Prompt for text input. On Windows/macOS/Linux, this will prompt on `stdout`. On iOS, this will prompt using a native dialog. 
    /// Currently not supported on WASM (but should use `window.prompt` in the future) and not supported on Android.
    pub fn prompt(&self, _message: &str) -> String {
        // temporary MacOS implementation using NSAlert
        #[cfg(target_os = "macos")]
        {
        // we need to use run_on_main_thread here because NSAlert is not thread safe

        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let alert = unsafe { NSAlert::new(mtm) };

        unsafe { alert.setMessageText(ns_string!("Please povide a subject id")) };
        // set button text
        unsafe { alert.addButtonWithTitle(ns_string!("OK")) };
        // set style to informational
        unsafe { alert.setAlertStyle(NSAlertStyle::Warning) };

        // add a text field
        let textfield =   MainThreadMarker::alloc(mtm);
        // initialize the textfield
        let rect = NSRect::new(CGPoint::new(0.0, 0.0), CGSize::new(200.0, 24.0));
        let textfield= unsafe { NSTextField::initWithFrame(textfield, rect) };

  
        let textfield_v = Id::into_super(textfield.clone());
        let textfield_v = Id::into_super(textfield_v);

        unsafe { alert.setAccessoryView(Some(&textfield_v)) };
        

        // show the alert
        let _response = unsafe { alert.runModal() };
    
        // get the text from the textfield
        let text = unsafe { textfield.stringValue() };
        let text = text.to_string();

        // return the text
        return text;
        }

        todo!();
    }


    /// Send a task to the render thread. The task will be executed on the render thread and will block the current thread until it is finished.
    pub fn run_on_main_thread<R, Fut>(
        &self,
        task: impl FnOnce() -> Fut + 'static + Send,
    ) -> R
    where
        Fut: Future<Output = R> + 'static + Send,
        R: Send + 'static,
    {
        // create channel to receive result
        let (tx, rx) = bounded(1);

        // create task
        let rtask = move || {
            let task = async move {
                let result = task().await;
                block_on(tx.send(result)).expect("Failed to send result to main thread. This is likely a bug, please report it.");
            };
            Box::pin(task) as Pin<Box<dyn Future<Output = ()> + Send>>
        };

        let rtask_boxed = Box::new(rtask)
            as Box<
                dyn FnOnce()
                        -> Pin<Box<dyn Future<Output = ()> + Send>>
                    + Send,
            >;

        // send task
        block_on(self.render_task_sender.send(rtask_boxed));

        // wait for result
        return block_on(rx.recv()).unwrap();
    }

    pub fn get_available_monitors(&mut self) -> Vec<Monitor> {
        let mut monitors = vec![];
        let event_loop = self.event_loop.as_ref().unwrap();
        for (i, handle) in event_loop.available_monitors().enumerate()
        {
            monitors.push(Monitor {
                name: handle
                    .name()
                    .unwrap_or(format!("Unnamed monitor {}", i)),
                handle: handle,
            });
        }
        monitors
    }

    /// Starts the experiment. This will block until the experiment is finished and exit the program afterwards.
    ///
    /// # Arguments
    ///
    /// * `experiment_fn` - The function that is your experiment. This function will be called with a `Window` object that you can use to create stimuli and submit frames to the window.
    pub fn run_experiment<F>(&mut self, experiment_fn: F) -> ()
    where
        F: FnOnce(WindowManager) -> Result<(), errors::PsychophysicsError>
            + 'static
            + Send,
    {
        let event_loop = self.event_loop.take().unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            smol::block_on(
                self.run_event_loop(event_loop, experiment_fn),
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            let winit_window =
                winit::window::Window::new(&event_loop).unwrap();
            std::panic::set_hook(Box::new(
                console_error_panic_hook::hook,
            ));
            console_log::init().expect("could not initialize logger");
            use winit::platform::web::WindowExtWebSys;
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(
                        winit_window.canvas(),
                    ))
                    .ok()
                })
                .expect("couldn't append canvas to document body");

            // set canvas size
            let _canvas = winit_window.canvas();
            let document =
                web_sys::window().unwrap().document().unwrap();
            let width = document.body().unwrap().client_width();
            let height = document.body().unwrap().client_height();
            winit_window.set_inner_size(
                winit::dpi::LogicalSize::new(
                    width as f64,
                    height as f64,
                ),
            );
            wasm_bindgen_futures::spawn_local(run(
                event_loop,
                winit_window,
                experiment_fn,
            ));
        }
    }

    async fn run_event_loop<F>(
        &mut self,
        event_loop: EventLoop<PsychophysicsEventLoopEvent>,
        experiment_fn: F,
    ) where
        F: FnOnce(WindowManager) -> Result<(), errors::PsychophysicsError>
            + 'static
            + Send,
    {
        log::debug!(
            "Main task is running on thread {:?}",
            std::thread::current().id()
        );

        let available_monitors = event_loop
            .available_monitors()
            .map(|monitor| Monitor {
                name: monitor.name().unwrap_or("Unnamed monitor".to_string()),
                handle: monitor,
            })
            .collect();

        let wm = WindowManager {
            event_loop_proxy: event_loop.create_proxy(),
            render_taks_sender: self.render_task_sender.clone(),
            available_monitors: available_monitors,
        };

        // // start renderer
        // {
        //     let win_handle = window.clone();
        //     #[cfg(target_arch = "wasm32")]
        //     spawn_async_task(render_task(window));
        //     #[cfg(not(target_arch = "wasm32"))]
        //     thread::spawn(move || {
        //         smol::block_on(render_task(win_handle));
        //     });
        // }

        // // start renderer2 on WASM, we run this async in the browsers event loop
        // // on native, we spawn a new thread (but we might need to change that)
        // {
        //     let win_handle = window.clone();
        //     #[cfg(target_arch = "wasm32")]
        //     spawn_async_task(render_task2(window));
        //     #[cfg(not(target_arch = "wasm32"))]
        //     let rpr = self.render_task_receiver.clone();
        //     thread::spawn(move || {
        //         smol::block_on(render_task2(rpr));
        //     });
        // }

        // start experiment
        thread::spawn(move || {
            let res = experiment_fn(wm);
            // panic if the experiment function returns an error
            if let Err(e) = res {
                // throw error
                log::error!("Experiment failed with {:?}: {:}", e, e);
                // quit program
                std::process::exit(1);
                
            }
        });

        // set event loop to poll
        event_loop.set_control_flow(ControlFlow::Poll);

        let _ =
            event_loop.run(move |event: WinitEvent<PsychophysicsEventLoopEvent>, win_target| {
                match event {
                    WinitEvent::UserEvent(event) => {
                        match event {
                            PsychophysicsEventLoopEvent::CreateNewWindowEvent(
                                window_options,
                                sender,
                            ) => {
                                log::debug!("Event loop received CreateNewWindowEvent - creating new window");

                                let window = self.create_window(
                                    &window_options,
                                    win_target,
                                );

                                      // // start renderer
                                {
                                    let win_handle = window.clone();
                                    #[cfg(target_arch = "wasm32")]
                                    spawn_async_task(render_task(window));
                                    #[cfg(not(target_arch = "wasm32"))]
                                    thread::spawn(move || {
                                        smol::block_on(render_task(win_handle.clone()));
                                    });
                                }

                                // push window to list of windows
                                self.windows.push(window.clone());
                                
                                sender.send_blocking(window).expect("Failed to send window to sender. This is likely a bug, please report it.");
                            },
                            PsychophysicsEventLoopEvent::PromptEvent(
                                message,
                                sender,
                            ) => {
                                log::debug!("Event loop received PromptEvent - showing prompt");

                                let result = self.prompt(&message);
                                
                                sender.send_blocking(result).expect("Failed to send result to sender. This is likely a bug, please report it.");
                            }
                            PsychophysicsEventLoopEvent::NewWindowCreatedEvent(
                                _window,
                            ) => {
                                log::debug!("Event loop received NewWindowCreatedEvent");
                                // add window to list of windows
                                //self.windows.push(window);
                            }
                            PsychophysicsEventLoopEvent::RunOnMainThread(
                                task,
                            ) => {
                                log::debug!("Running task on main thread");
                                let _ = block_on(task());
                            }
                        }
                    }
                    WinitEvent::WindowEvent {
                        window_id: id,
                        event: WindowEvent::Resized(new_size),
                    } => {
                        log::debug!(
                            "Window {:?} resized to {:?}",
                            id,
                            new_size
                        );
            
                        if let  Some(window) = self.get_window_by_id(id) {
                        //
                        let mut window_state =
                            window.write_window_state_blocking();
                            let gpu_state = self.gpu_state.read_blocking();
                        window_state.config.width = new_size.width.max(1);
                        window_state.config.height =
                            new_size.height.max(1);
                        window_state.surface.configure(
                            &gpu_state.device,
                            &window_state.config,
                        );

                        // on macos, the window size is not updated automatically
                        window_state.window.request_redraw();

                        // update window size
                        window.width_px.store(
                            new_size.width as u32,
                            Ordering::Relaxed,
                        );
                        window.height_px.store(
                            new_size.height as u32,
                            Ordering::Relaxed,
                        );
                    }
                }
                  
                    // handle window events
                    WinitEvent::WindowEvent {
                        window_id: id,
                        event,
                    } => {

                        
                        
                        if let  Some(window) = self.get_window_by_id(id) {    
                                if let Some(input) =
                                    Event::try_from(event.clone()).ok()
                                {
                
                                    // if escape key was pressed, close window
                                    if input.key_pressed("\u{1b}") {
                                        win_target.exit();
                                    }

                                    let physical_input_sender =
                                        &window.physical_input_sender;
                                    let _ = physical_input_sender
                                        .try_broadcast(input);
                                }
                            }
                    }
                    // handle close event
                    _ => {}
                }
            });
    }

    pub fn get_window_by_id(&self, id: winit::window::WindowId) -> Option<Window> {
        for window in &self.windows {
            if window.read_window_state_blocking().window.id() == id {
                return Some(window.clone());
            }
        }
        None
    }
}

/// This is the second render task. It is used to execute tasks on the render thread when running on WASM.
async fn render_task2(
    render_task_receiver: Receiver<
        Box<
            dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send,
        >,
    >,
) {
    log::debug!(
        "Render task 2 running on thread {:?}",
        std::thread::current().id()
    );
    // loop forever
    loop {
        // wait until task is available
        let task = render_task_receiver.recv().await.unwrap();
        // await the task (the task itself will handle the backchannel)
        task().await;
    }
}

#[macro_export]
/// This is a convenience macro for running a loop that submits frames to the window.
/// It takes a window and a body of code that is executed on every iteration of the loop.
/// The loop will run until you `break` out of it, the provided timeout is reached,
/// or one of the provided keys is pressed.
///
/// # Example
///
/// ```no_run
/// loop_frames!(frame from window, keys = Key::Escape, {
///    // set frame color to white
///    frame.set_bg_color(color::WHITE);
///    // add stimuli to frame
///    frame.add(&my_stimulus);
/// });
/// ```
///
/// # Notes about delays
///
/// Every iteration of the loop, the macro will fetch a new frame from the window,
/// run the body of code, and submit the frame to the window. Note that this approach
/// blocks until the frame is submitted to the window. This also means that you handling
/// events will incur a delay of up to one frame. If you want to handle events without
/// this kind of delay, consider using a callback function instead. Callbacks will be run
/// within the main render loop, so they will not suffer from this kind of delay.
///
/// Alternatively, you can also fetch frames from the window manually using the `get_frame_try` method,
/// which will only return a frame if one is available but will not block otherwise. This will also work
/// in a multi-window setup where you want to submit frames to different windows within the same loop,
/// even if the windows have different refresh rates/and or run out-of-sync.
///
/// Lastly, you can opt to spawn a separate thread that submits frames to the window. However, this
/// means that you have to take care of synchronisation yourself, e.g., by using channels or by putting
/// any shared data behind a mutex.
macro_rules! loop_frames {
    ( ($frame_i:ident, $frame:ident) from $win:expr $(, keys = $keys:expr)?  $(, timeout = $timeout:expr)?, $body:block) => {
        {

            use $crate::input::Key;

            let timeout_duration = $(Some(web_time::Duration::from_secs_f64($timeout));)? None as Option<web_time::Duration>;

            let key = $(Some($keys);)? None as Option<Key>;

            //let keys_vec: Vec<Key> = $($keys.into_iter().map(|k| k.into()).collect();)? vec![] as Vec<Key>;

            let mut keyboard_receiver = $win.physical_input_receiver.activate_cloned();
            let mut $frame_i = 0;

            let kc: Option<Key> = None;

            {
                let mut $frame = $win.get_frame();
                $body
                $win.submit_frame($frame);
            }

            let start = web_time::Instant::now();

            'outer: loop {

                // check if timeout has been reached
                if timeout_duration.is_some() && start.elapsed() > timeout_duration.unwrap() {
                    break 'outer;
                }
                // check if a key has been pressed
                while let Ok(e) = keyboard_receiver.try_recv() {
                    // check if the key is the one we are looking for
                    if let Some(k) = key {
                        if e.key_pressed(k) {
                            break 'outer;
                        }
                    }
                }
                // if not, run another iteration of the loop
                let mut $frame = $win.get_frame();
                $frame_i = $frame_i + 1;
                $body
                $win.submit_frame($frame);
            }
        (kc, start.elapsed())
        }
    };

    ($frame:ident from $win:expr $(, keys = $keys:expr)? $(, keystate = $keystate:expr)? $(, timeout = $timeout:expr)?, $body:block) => {
       { // call loop_frames! macro with default iteration variable name
        let _frame_i = 0;
        loop_frames!((_frame_i, $frame) from $win $(, keys = $keys)? $(, keystate = $keystate)? $(, timeout = $timeout)?, $body)
         }
    };
}
