#![recursion_limit = "512"]
use async_broadcast::broadcast;
use async_channel::{bounded, Receiver, Sender};
use async_lock::Mutex;

use atomic_float::AtomicF64;
use futures_lite::Future;

use crate::visual::color::ColorFormat;

use crate::utils::BlockingLock;
use async_executor::Executor;

use input::Key;
use wgpu::TextureFormat;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use wasm_bindgen::{closure::Closure, JsCast};
use web_time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

// this is behind a feature flag because it is not yet stable
#[cfg(feature = "gst")]
pub mod camera;
pub mod errors;
pub mod input;
pub mod onnx;
pub mod utils;
pub mod visual;
use winit::event_loop::EventLoop;

// the prelude
pub mod prelude {
    pub use crate::errors::PsychophysicsError;
    pub use crate::input::KeyPressReceiver;
    pub use crate::utils::sleep_secs;
    pub use crate::utils::BIDSEventLogger;
    pub use crate::visual::geometry::{Rectangle, Size};
    pub use crate::visual::stimuli::GratingsStimulus;
    pub use crate::visual::{stimuli::TextStimulus, Window};
    pub use crate::{loop_frames, start_experiment};
}

#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;

use crate::visual::window::{
    render_task, render_task2, Frame, Window, WindowState,
};
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

impl UnwrapKeyPressAndDuration
    for Result<PFutureReturns, anyhow::Error>
{
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

// pub async fn async_sleep(
//     secs: f64,
// ) -> Result<PFutureReturns, anyhow::Error> {
//     let start = web_time::Instant::now();
//     #[cfg(not(target_arch = "wasm32"))]
//     smol::Timer::after(Duration::from_micros(
//         (secs * 1000000.0) as u64,
//     ))
//     .await;
//     #[cfg(target_arch = "wasm32")]
//     {
//         let window = web_window();
//         let promise = js_sys::Promise::new(
//             &mut |resolve, _reject| {
//                 window.set_timeout_with_callback_and_timeout_and_arguments_0(
//                 &resolve,
//                 (secs * 1000.0) as i32,
//             );
//             },
//         );
//         wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
//     }
//     let end = web_time::Instant::now();
//     return Ok(PFutureReturns::Timeout(end.duration_since(start)));
// }

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

pub fn start_experiment<F>(experiment_fn: F) -> ()
where
    F: FnOnce(Window) -> Result<(), errors::PsychophysicsError>
        + 'static
        + Send,
{
    let event_loop = EventLoop::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        simple_logger::SimpleLogger::new().env().init().unwrap();
        log::set_max_level(log::LevelFilter::Info);
        // get monitor
        let monitor = event_loop.available_monitors().nth(1).unwrap_or_else(|| {
            println!("No secondary monitor found, using primary monitor");
            event_loop
                .primary_monitor()
                .expect("No primary monitor found")
        });

        log::info!("Monitor informaton: {:?}", monitor);

        // find video mode with resoltion that matches the monitor size
        let video_modes =
            monitor.video_modes().filter(|video_mode| {
                video_mode.size().width as u32
                    == monitor.size().width as u32
                    && video_mode.size().height as u32
                        == monitor.size().height as u32
            });

        let video_mode = video_modes
            .filter(|video_mode| {
                video_mode.refresh_rate_millihertz() == 120_000
            })
            .max_by_key(|video_mode| {
                video_mode.refresh_rate_millihertz()
            })
            .expect("Could not find a suitable video mode");

        let true_size = video_mode.size();

        log::info!("Selected video mode: {:?}", video_mode);

        let winit_window = winit::window::WindowBuilder::new()
            // make exclusive fullscreen
            .with_fullscreen(Some(
                winit::window::Fullscreen::Exclusive(video_mode),
            ))
            .with_title("Metal".to_string())
            .with_inner_size(true_size)
            .build(&event_loop)
            .unwrap();

        // hide cursor
        winit_window.set_cursor_visible(false);

        // run
        smol::block_on(run(event_loop, winit_window, experiment_fn));
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
        let document = web_sys::window().unwrap().document().unwrap();
        let width = document.body().unwrap().client_width();
        let height = document.body().unwrap().client_height();
        winit_window.set_inner_size(winit::dpi::LogicalSize::new(
            width as f64,
            height as f64,
        ));
        wasm_bindgen_futures::spawn_local(run(
            event_loop,
            winit_window,
            experiment_fn,
        ));
    }
}

async fn run<F>(
    event_loop: EventLoop<()>,
    winit_window: winit::window::Window,
    experiment_fn: F,
) where
    F: FnOnce(Window) -> Result<(), errors::PsychophysicsError>
        + 'static
        + Send,
{
    log::debug!(
        "Main task is running on thread {:?}",
        std::thread::current().id()
    );

    let size = winit_window.inner_size();

    let instance = wgpu::Instance::default();

    let surface =
        unsafe { instance.create_surface(&winit_window) }.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropiate graphics adapter. This is likely a bug, please report it.");

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
        .expect("Failed to create device. This is likely a bug, please report it.");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = TextureFormat::Bgra8Unorm;
    let swapchain_view_format = vec![
        TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb,
    ];

    // log supported texture formats
    log::info!("Supported texture formats:");
    for format in swapchain_capabilities.formats {
        log::info!("{:?}", format);
    }

    log::info!("Selected swapchain format: {:?}", swapchain_format);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: swapchain_view_format,
    };

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

    // create broadcast channel
    let mut keyboard_sender: async_broadcast::Sender<
        winit::event::KeyboardInput,
    >;
    let keyboard_receiver: async_broadcast::Receiver<
        winit::event::KeyboardInput,
    >;
    (keyboard_sender, keyboard_receiver) = broadcast(100);

    // create channel for sending tasks to the render thread
    let (render_task_sender, render_task_receiver) = bounded(100);

    // set overflow strategy
    keyboard_sender.set_overflow(true);

    let keyboard_receiver = keyboard_receiver.deactivate();

    // create a pwindow
    let pwindow = WindowState {
        window: winit_window,
        event_loop_proxy: event_loop.create_proxy(),
        device,
        instance,
        surface,
        adapter,
        queue,
        config,
    };

    // create handle
    let win_handle = Window {
        state: Arc::new(Mutex::new(pwindow)),
        keyboard_receiver,
        frame_sender,
        frame_receiver,
        frame_ok_sender,
        frame_ok_receiver,
        physical_width: Arc::new(AtomicF64::new(300.0)),
        viewing_distance: Arc::new(AtomicF64::new(57.0)),
        color_format: ColorFormat::SRGBA8,
        render_task_sender: render_task_sender,
        render_task_receiver: render_task_receiver,
        width_px: Arc::new(AtomicU32::new(300)),
        height_px: Arc::new(AtomicU32::new(300)),
    };

    // start renderer
    {
        let win_handle = win_handle.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_async_task(render_task(win_handle));
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            smol::block_on(render_task(win_handle));
        });
    }
    // start renderer2
    {
        let win_handle = win_handle.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_async_task(render_task2(win_handle));
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            smol::block_on(render_task2(win_handle));
        });
    }

    let cwh = win_handle.clone();

    // start experiment
    thread::spawn(move || {
        let res = experiment_fn(cwh.clone());
        if let Err(e) = res {
            log::error!("Experiment failed with error: {}", e);
            errors::show_error_screen(&cwh, e);
        }
    });

    event_loop.run(move |event: Event<'_, ()>, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                log::info!("Window resized with size {:?}", new_size);
                // Reconfigure the surface with the new size (this should likely be done on the renderer thread instead)
                let mut pwindow = win_handle.state.lock_blocking();
                pwindow.config.width = new_size.width.max(1);
                pwindow.config.height = new_size.height.max(1);
                pwindow
                    .surface
                    .configure(&pwindow.device, &pwindow.config);

                // update window size
                win_handle
                    .width_px
                    .store(new_size.width as u32, Ordering::Relaxed);
                win_handle
                    .height_px
                    .store(new_size.height as u32, Ordering::Relaxed);
            }
            Event::UserEvent(()) => {
                // close window
                *control_flow = ControlFlow::Exit;
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
                        winit::event::VirtualKeyCode::Escape => {
                            *control_flow = ControlFlow::Exit
                        }
                        // send keypresses to channel

                        // log any other keypresses
                        _ => {
                            let _ =
                                keyboard_sender.try_broadcast(input);
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

#[macro_export]
macro_rules! loop_frames {
    ( ($frame_i:ident, $frame:ident) from $win:expr $(, keys = $keys:expr)? $(, keystate = $keystate:expr)? $(, timeout = $timeout:expr)?, $body:block) => {
        {

            use $crate::input::Key;
            use $crate::input::KeyState;

            let timeout_duration = $(Some(web_time::Duration::from_secs_f64($timeout));)? None as Option<web_time::Duration>;

            let keys_vec: Vec<Key> = $($keys.into_iter().map(|k| k.into()).collect();)? vec![] as Vec<Key>;
            let keystate: KeyState = $($keystate;)? KeyState::Pressed;

            let mut keyboard_receiver = $win.keyboard_receiver.activate_cloned();
            let mut $frame_i = 0;

            let mut kc: Option<Key> = None;
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
                    // check if the key is one of the keys we are looking for
                    if keys_vec.contains(&e.virtual_keycode.unwrap().into()) && keystate == e.state.into() {
                        kc = Some(e.virtual_keycode.unwrap().clone().into());
                        break 'outer;
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
