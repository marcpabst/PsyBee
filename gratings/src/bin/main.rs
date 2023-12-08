use async_broadcast::broadcast;
use async_std::sync::Mutex;
use async_std::task::{self};
use futures_lite::future::block_on;
use gratings::input::Key;

use flume::{bounded, Receiver, Sender};
use futures_lite::FutureExt;
use gratings::visual::Color;
use rodio::source::Spatial;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

use web_time::Duration;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
extern crate gratings;
use gratings::visual::gratings::GratingsStimulus;
use gratings::visual::pwindow::PWindow;
use gratings::visual::text::{TextStimulus, TextStimulusConfig};
use gratings::visual::Renderable;

// macro to log to sdout or console, depending on target
macro_rules! log_extra {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($arg)*);
        }
        #[cfg(target_arch = "wasm32")]
        {
            console::log_1(&format!($($arg)*).into());
        }
    };
}

enum PFutureReturns {
    Duration(Duration),
    Timeout(Duration),
    KeyPress((Key, Duration)),
    NeverReturn,
}

// implement unwrap_duration for Result<PFutureReturns, anyhow::Error>
trait UnwrapDuration {
    fn unwrap_duration(self) -> Duration;
    fn is_duration(&self) -> bool;
}
trait UnwrapKeyPressAndDuration {
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

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
pub struct Frame {
    renderables: Arc<Mutex<Vec<Box<dyn Renderable>>>>,
    bg_color: wgpu::Color,
}

impl Renderable for Frame {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) -> () {
        // call prepare() on all renderables
        for renderable in &mut (block_on(self.renderables.lock())).iter_mut() {
            renderable.prepare(device, queue, view, config);
        }
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // call render() on all renderables
        for renderable in &mut (block_on(self.renderables.lock())).iter_mut() {
            renderable.render(enc, view);
        }
    }
}

impl Frame {
    // create a new frame
    pub fn new() -> Self {
        Self {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color: Color::RGB {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }
            .into(),
        }
    }

    pub fn new_with_bg_color(bg_color: wgpu::Color) -> Self {
        Self {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color,
        }
    }

    // add a renderable to the frame
    pub fn add(&mut self, renderable: &(impl Renderable + Clone + 'static)) -> () {
        let renderable = Box::new(renderable.clone());
        block_on(self.renderables.lock()).push(renderable);
    }
}

// mark Frame as Send and Sync
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

async fn submit_frame(frame: Frame, tx: Sender<Arc<Mutex<Frame>>>, rx: Receiver<bool>) {
    // submit frame to channel

    let _ = tx.send_async(Arc::new(Mutex::new(frame))).await;
    // wait for frame to be consumed
    let _ = rx.recv_async().await;
    //console::log_1(&"Frame consumed".into());
}

async fn sleep(secs: f64) -> Result<PFutureReturns, anyhow::Error> {
    let start = web_time::Instant::now();
    async_std::task::sleep(Duration::from_micros((secs * 1000000.0) as u64)).await;
    let end = web_time::Instant::now();
    return Ok(PFutureReturns::Duration(end.duration_since(start)));
}

async fn wait_for_keypress<T, I>(
    // the PWindow behind a mutex
    pwindow: Arc<Mutex<PWindow>>,
    keys: T,
) -> Result<PFutureReturns, anyhow::Error>
where
    T: IntoIterator<Item = I>,
    I: Into<Key>,
{
    let start = web_time::Instant::now();
    let mut keyboard_receiver;
    {
        // create new receiver
        let pwindow = pwindow.lock().await;
        keyboard_receiver = pwindow.keyboard_receiver.activate_cloned();
    }

    let key_vec: Vec<Key> = keys.into_iter().map(|k| k.into()).collect();

    loop {
        // wait for buttons pres
        let e = keyboard_receiver
            .recv()
            .await
            .map_err(|_| anyhow::anyhow!("Failed to receive keypress from channel"))?;

        // check if keypress matches any of the keys
        if key_vec.contains(&e.virtual_keycode.unwrap().into()) || key_vec.is_empty() {
            break;
        }
    }

    return Ok(PFutureReturns::KeyPress((
        Key::Space,
        web_time::Instant::now().duration_since(start),
    )));
}

async fn wait_for_any_keypress(
    pwindow: Arc<Mutex<PWindow>>,
) -> Result<PFutureReturns, anyhow::Error> {
    let empty_vec: Vec<Key> = Vec::new();
    return wait_for_keypress(pwindow, empty_vec).await;
}

async fn render_task(
    rx_pwindow: Receiver<Arc<Mutex<PWindow>>>,
    rx: Receiver<Arc<Mutex<Frame>>>,
    tx: Sender<bool>,
) {
    //  on wasm, we register our own requestAnimationFrame callback in a separate task
    #[cfg(target_arch = "wasm32")]
    {
        log_extra!(
            "Task RENDER running on thread {:?}",
            std::thread::current().id()
        );

        // get window from channel
        let win = rx_pwindow.recv_async().await.unwrap();

        // here, we create a closure that will be called by requestAnimationFrame
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            // Set the body's text content to how many times this
            // requestAnimationFrame callback has fired.

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());

            // check if there is a frame available
            let try_frame = rx.try_recv();

            if try_frame.is_ok() {
                let frame = try_frame.unwrap();
                // acquire lock on frame
                let mut frame = block_on(frame.lock());

                // acquire lock on window
                let window_lock = block_on(win.lock());

                let suface_texture: wgpu::SurfaceTexture = window_lock
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = suface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = window_lock
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // clear the frame
                {
                    // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                    // using the LoadOp::Clear option)
                    let rpass = &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(frame.bg_color.into()),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                frame.prepare(
                    &window_lock.device,
                    &window_lock.queue,
                    &view,
                    &window_lock.config,
                );

                frame.render(&mut encoder, &view);

                window_lock.queue.submit(Some(encoder.finish()));
                suface_texture.present();

                // notify sender that frame has been consumed
                let _ = tx.try_send(true);
            }
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
    // on native, we submit frames when they are ready
    #[cfg(not(target_arch = "wasm32"))]
    {
        log_extra!(
            "Task RENDER running on thread {:?}",
            std::thread::current().id()
        );

        // get window from channel
        let win = rx_pwindow.recv_async().await.unwrap();

        loop {
            // wait for frame to be submitted
            let frame = rx.recv_async().await.unwrap();

            // acquire lock on frame
            let mut frame = block_on(frame.lock());

            // acquire lock on window
            let window_lock = block_on(win.lock());

            let suface_texture = window_lock
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture");
            let view = suface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = window_lock
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            // clear the frame
            {
                // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                // using the LoadOp::Clear option)
                let _rpass = &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(frame.bg_color.into()),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            frame.prepare(
                &window_lock.device,
                &window_lock.queue,
                &view,
                &window_lock.config,
            );
            frame.render(&mut encoder, &view);

            window_lock.queue.submit(Some(encoder.finish()));
            suface_texture.present();

            // notify sender that frame has been consumed
            let _ = tx.send_async(true).await;
        }
    }
}

async fn test(
    tx: Sender<Arc<Mutex<Frame>>>,
    rx: Receiver<bool>,
    rx_pwindow: Receiver<Arc<Mutex<PWindow>>>,
) {
    log_extra!(
        "Task TEST running on thread {:?}",
        std::thread::current().id()
    );

    // wait for window to be ready
    let win = rx_pwindow.recv_async().await.unwrap();

    // create all text stimuli
    let start_text = TextStimulus::new(
        &block_on(win.lock()),
        TextStimulusConfig {
            text: "Press space to start".to_string(),
            color: Color::RGB {
                r: 0.0,
                g: 255.0,
                b: 0.0,
            }
            .into(),
            ..Default::default()
        },
    );

    // create a word text
    let mut word_text = TextStimulus::new(
        &block_on(win.lock()),
        TextStimulusConfig {
            text: "WORD".to_string(),
            color: Color::RGB {
                r: 0.0,
                g: 255.0,
                b: 0.0,
            }
            .into(),
            ..Default::default()
        },
    );

    let end_text = TextStimulus::new(
        &block_on(win.lock()),
        TextStimulusConfig {
            text: "End of experiment".to_string(),
            color: Color::RGB {
                r: 0.0,
                g: 255.0,
                b: 0.0,
            }
            .into(),
            ..Default::default()
        },
    );

    // create a fixation cross
    let fixation_cross = TextStimulus::new(
        &block_on(win.lock()),
        TextStimulusConfig {
            text: "+".to_string(),
            ..Default::default()
        },
    );

    let start_screen = async {
        loop {
            let mut frame = Frame::new_with_bg_color(wgpu::Color::BLACK);
            // add text stimulus to frame
            frame.add(&start_text);
            // submit frame
            submit_frame(frame, tx.clone(), rx.clone()).await;
        }
    };

    // show start screen until SPACE is pressed
    start_screen
        .or(wait_for_keypress(win.clone(), Key::Space))
        .await;

    for i in 0..10 {
        let fixiation_screen = async {
            loop {
                let mut frame = Frame::new();
                // add fixation cross to frame
                frame.add(&fixation_cross);
                // submit frame
                submit_frame(frame, tx.clone(), rx.clone()).await;
            }
        };

        // show fixiation screen for 50ms second
        fixiation_screen.or(sleep(0.5)).await;

        let word_screen = async {
            // create a random color
            word_text.set_color(
                Color::RGB {
                    r: rand::random::<f64>(),
                    g: rand::random::<f64>(),
                    b: rand::random::<f64>(),
                }
                .into(),
            );

            loop {
                let mut frame = Frame::new();
                // add word text to frame
                frame.add(&word_text);
                // submit frame
                submit_frame(frame, tx.clone(), rx.clone()).await;
            }

            // this is never reached but informs the compiler about the return type
            return Result::<PFutureReturns, anyhow::Error>::Ok(PFutureReturns::NeverReturn);
        };

        // show word screen for 500ms or until either R, G, B is pressed
        let res = word_screen
            .or(sleep(2.0))
            .or(wait_for_keypress(win.clone(), vec![Key::R, Key::G, Key::B]))
            .await;

        if res.is_keypress() {
            let (key, duration) = res.unwrap_key_and_duration();
            log_extra!("Keypress {:?} after {:?}", key, duration);
        } else {
            let duration = res.unwrap_duration();
            log_extra!("Timeout after {:?}", duration);
        }
    }
    // show end screen
    loop {
        let mut frame = Frame::new_with_bg_color(wgpu::Color::BLACK);
        // add text stimulus to frame
        frame.add(&end_text);
        // submit frame
        submit_frame(frame, tx.clone(), rx.clone()).await;
    }
}

async fn run(
    event_loop: EventLoop<()>,
    window: Window,
    _tx: Sender<bool>,
    _rx: Receiver<Arc<Mutex<Frame>>>,
    tx_pwindow: Sender<Arc<Mutex<PWindow>>>,
) {
    log_extra!(
        "Task RUN running on thread {:?}",
        std::thread::current().id()
    );

    let size = window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
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

    // create broadcast channel
    let mut keyboard_sender: async_broadcast::Sender<winit::event::KeyboardInput>;
    let keyboard_receiver: async_broadcast::Receiver<winit::event::KeyboardInput>;
    (keyboard_sender, keyboard_receiver) = broadcast(100);

    // set overflow strategy
    keyboard_sender.set_overflow(true);

    let keyboard_receiver = keyboard_receiver.deactivate();

    // create a pwindow
    let pwindow: Arc<Mutex<PWindow>> = Arc::new(Mutex::new(PWindow {
        window,
        device,
        instance,
        surface,
        adapter,
        queue,
        config,
        keyboard_receiver: keyboard_receiver.clone(),
    }));

    // submit pwindow to channel
    let _ = tx_pwindow.send_async(pwindow.clone()).await;
    let _ = tx_pwindow.send_async(pwindow.clone()).await;
    let _ = tx_pwindow.send_async(pwindow).await;

    event_loop.run(move |event: Event<'_, ()>, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.

        // let device = pwindow.device.lock().block_on();
        // let surface = pwindow.surface.lock().block_on();
        // let mut config = pwindow.config.lock().block_on();
        // let queue = pwindow.queue.lock().block_on();
        // let window = pwindow.window.lock().block_on();

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

fn main() {
    // create channels
    let (tx1, rx1): (Sender<Arc<Mutex<Frame>>>, Receiver<Arc<Mutex<Frame>>>) = bounded(1);
    let (tx2, rx2): (Sender<bool>, Receiver<bool>) = bounded(1);
    let (tx3, rx3): (Sender<Arc<Mutex<PWindow>>>, Receiver<Arc<Mutex<PWindow>>>) = bounded(1);
    //let (tx_events, rx_events): (Sender<Event>, Receiver<Event>) = bounded(1000);

    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // sleep for 1 second to make sure window is ready

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
        let _video_mode = monitor
            .video_modes()
            .filter(|m| m.size() == target_size)
            .max_by_key(|m| m.refresh_rate_millihertz())
            .unwrap();

        // make fullscreen
        //window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
        env_logger::init(); // Enable logging

        // run using async_std
        task::spawn(test(tx1.clone(), rx2.clone(), rx3.clone()));
        task::spawn(render_task(rx3.clone(), rx1.clone(), tx2.clone()));
        task::block_on(run(
            event_loop,
            window,
            tx2.clone(),
            rx1.clone(),
            tx3.clone(),
        ));
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
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(
            event_loop,
            window,
            tx2.clone(),
            rx1.clone(),
            tx3.clone(),
        ));
        wasm_bindgen_futures::spawn_local(render_task(rx3.clone(), rx1.clone(), tx2.clone()));
        wasm_bindgen_futures::spawn_local(test(tx1, rx2, rx3));
    }
}
