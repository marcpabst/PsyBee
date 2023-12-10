use async_broadcast::broadcast;
use async_channel::{bounded, Receiver, Sender};
use async_lock::Mutex;

use futures_lite::{future::block_on, Future};

use async_executor::Executor;

use input::Key;

use std::sync::Arc;

use wasm_bindgen::{closure::Closure, JsCast};
use web_time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

pub mod input;
pub mod visual;
use winit::{event_loop::EventLoop, window::Window};

use crate::visual::pwindow::{Frame, PWindow, WindowHandle};
pub enum PFutureReturns {
    Duration(Duration),
    Timeout(Duration),
    KeyPress((Key, Duration)),
    NeverReturn,
}

// implement unwrap_duration for Result<PFutureReturns, anyhow::Error>
pub trait UnwrapDuration {
    fn unwrap_duration(self) -> Duration;
    fn is_duration(&self) -> bool;
    fn is_timeout(&self) -> bool;
}
pub trait UnwrapKeyPressAndDuration {
    fn unwrap_key_and_duration(self) -> (Key, Duration);
    fn is_keypress(&self) -> bool;
}

impl UnwrapDuration for Result<PFutureReturns, anyhow::Error> {
    fn unwrap_duration(self) -> Duration {
        match self {
            Ok(PFutureReturns::Duration(d)) => d,
            Ok(PFutureReturns::Timeout(d)) => d,
            Ok(PFutureReturns::KeyPress((_, d))) => d,
            Ok(PFutureReturns::NeverReturn) => {
                panic!("unwrap_duration() called on PFutureReturns::NeverReturn. this should never happen.")
            }
            Err(_) => {
                // panick with error
                panic!("unwrap_duration() called on an Err value.")
            }
        }
    }
    fn is_duration(&self) -> bool {
        match self {
            Ok(PFutureReturns::Duration(_)) => true,
            Ok(PFutureReturns::Timeout(_)) => true,
            _ => false,
        }
    }

    fn is_timeout(&self) -> bool {
        match self {
            Ok(PFutureReturns::Timeout(_)) => true,
            _ => false,
        }
    }
}

impl UnwrapKeyPressAndDuration for Result<PFutureReturns, anyhow::Error> {
    fn unwrap_key_and_duration(self) -> (Key, Duration) {
        match self {
            Ok(PFutureReturns::KeyPress((k, d))) => (k, d),
            Ok(PFutureReturns::NeverReturn) => {
                panic!("unwrap_duration() called on PFutureReturns::NeverReturn. this should never happen.")
            }
            Err(_) => {
                // panick with error
                panic!("unwrap_keypress() called on an Err value.")
            }
            _ => {
                panic!("unwrap_keypress() called on a non-keypress value.")
            }
        }
    }
    fn is_keypress(&self) -> bool {
        match self {
            Ok(PFutureReturns::KeyPress(_)) => true,
            _ => false,
        }
    }
}

pub async fn sleep(secs: f64) -> Result<PFutureReturns, anyhow::Error> {
    let start = web_time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    smol::Timer::after(Duration::from_micros((secs * 1000000.0) as u64)).await;
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_window();
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                (secs * 1000.0) as i32,
            );
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }
    let end = web_time::Instant::now();
    return Ok(PFutureReturns::Timeout(end.duration_since(start)));
}

// macro to log to sdout or console, depending on target
#[macro_export]
macro_rules! log_extra {

    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($arg)*);
        }
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!($($arg)*).into());
        }
    };
}

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

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    log_extra!(
        "Task SPAWN running on thread {:?}",
        std::thread::current().id()
    );
    smol::spawn(future).detach();
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_task<F>(future: F)
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
pub trait FutureReturnTrait: Future<Output = ()> + 'static + Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<F> FutureReturnTrait for F where F: Future<Output = ()> + 'static + Send {}
#[cfg(target_arch = "wasm32")]
pub trait FutureReturnTrait: Future<Output = ()> + 'static {}
#[cfg(target_arch = "wasm32")]
impl<F> FutureReturnTrait for F where F: Future<Output = ()> + 'static {}

pub fn start_experiment<F>(experiment_fn: impl FnOnce(WindowHandle) -> F + 'static)
where
    F: FutureReturnTrait,
{
    let event_loop = EventLoop::new();
    let winit_window = winit::window::Window::new(&event_loop).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // get monitor
        let monitor = winit_window.available_monitors().nth(1).unwrap_or_else(|| {
            println!("No second monitor found, using current monitor");
            winit_window.current_monitor().unwrap()
        });

        // get video mode with biggest width
        let target_size = monitor
            .video_modes()
            .max_by_key(|m| m.size().width)
            .unwrap()
            .size();

        // get video mode with biggest width and highest refresh rate
        let _video_mode = monitor
            .video_modes()
            .filter(|m| m.size() == target_size)
            .max_by_key(|m| m.refresh_rate_millihertz())
            .unwrap();

        // make fullscreen
        //window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
        env_logger::init(); // Enable logging

        // run
        block_on(run(event_loop, winit_window, experiment_fn));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(winit_window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, winit_window, experiment_fn));
    }
}

async fn run<F>(
    event_loop: EventLoop<()>,
    winit_window: Window,
    experiment_fn: impl FnOnce(WindowHandle) -> F,
) where
    F: FutureReturnTrait,
{
    log_extra!(
        "Task RUN running on thread {:?}",
        std::thread::current().id()
    );

    let size = winit_window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&winit_window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
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

    // create channel for frame submission
    let (frame_sender, frame_receiver): (Sender<Arc<Mutex<Frame>>>, Receiver<Arc<Mutex<Frame>>>) =
        bounded(1);

    let (frame_ok_sender, frame_ok_receiver): (Sender<bool>, Receiver<bool>) = bounded(1);

    // create broadcast channel
    let mut keyboard_sender: async_broadcast::Sender<winit::event::KeyboardInput>;
    let keyboard_receiver: async_broadcast::Receiver<winit::event::KeyboardInput>;
    (keyboard_sender, keyboard_receiver) = broadcast(100);

    // set overflow strategy
    keyboard_sender.set_overflow(true);

    let keyboard_receiver = keyboard_receiver.deactivate();

    // create a pwindow
    let pwindow = PWindow {
        window: winit_window,
        device,
        instance,
        surface,
        adapter,
        queue,
        config,
    };

    // create handle
    let win_handle = WindowHandle {
        pw: Arc::new(Mutex::new(pwindow)),
        keyboard_receiver,
        frame_sender,
        frame_receiver,
        frame_ok_sender,
        frame_ok_receiver,
    };

    // start renderer
    spawn_task(win_handle.clone().render_task());

    // start experiment
    spawn_task(experiment_fn(win_handle.clone()));

    event_loop.run(move |event: Event<'_, ()>, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                // Reconfigure the surface with the new size
                // config.width = size.width;
                // config.height = size.height;
                // surface.configure(&device, &config);
                // // On macos the window needs to be redrawn manually after resizing
                // window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // nothing to do here
                // on web, we register our own requestAnimationFrame callback in a separate thread
                // on native, we submit frames to the channel in a separate thread
            }
            // handle keyboard input
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        winit::event::VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        // send keypresses to channel

                        // log any other keypresses
                        _ => {
                            let _ = keyboard_sender.try_broadcast(input);
                        }
                    }
                }
            }
            // handle close event
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
