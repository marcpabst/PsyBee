use async_broadcast::broadcast;
use async_std::sync::Mutex;
use async_std::task::{self};
use futures_lite::future::block_on;
use gratings::input::Key;

use gratings::{sleep, spawn_task, PFutureReturns, UnwrapDuration, UnwrapKeyPressAndDuration};

use flume::{bounded, Receiver, Sender};
use futures_lite::FutureExt;
use gratings::visual::Color;
use rodio::source::Spatial;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_time::Duration;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
extern crate gratings;
use gratings::log_extra;
use gratings::visual::pwindow::{Frame, PWindow, WindowHandle};
use gratings::visual::text::{TextStimulus, TextStimulusConfig};
use gratings::visual::Renderable;

async fn experiment(window: WindowHandle) {
    log_extra!(
        "Task TEST running on thread {:?}",
        std::thread::current().id()
    );

    // define colors for stroop task
    let COLORS = vec![
        Color::RGB {
            r: 255.0,
            g: 0.0,
            b: 0.0,
        },
        Color::RGB {
            r: 0.0,
            g: 255.0,
            b: 0.0,
        },
        Color::RGB {
            r: 0.0,
            g: 0.0,
            b: 255.0,
        },
    ];

    // create all text stimuli
    let start_text = TextStimulus::new(
        &window,
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
        &window,
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
        &window,
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
        &window,
        TextStimulusConfig {
            text: "+".to_string(),
            ..Default::default()
        },
    );

    log_extra!("Created all stimuli");

    let start_screen = async {
        loop {
            let mut frame = Frame::new_with_bg_color(wgpu::Color::BLACK);
            // add text stimulus to frame
            frame.add(&start_text);
            // submit frame
            window.submit_frame(frame).await;
        }
    };

    // show start screen until SPACE is pressed
    start_screen.or(window.wait_for_keypress(Key::Space)).await;

    for i in 0..10 {
        let fixiation_screen = async {
            loop {
                let mut frame = Frame::new();
                // add fixation cross to frame
                frame.add(&fixation_cross);
                // submit frame
                window.submit_frame(frame).await;
            }
        };

        // show fixiation screen for 50ms second
        fixiation_screen.or(sleep(0.5)).await;

        let word_screen = async {
            // create a random color
            word_text.set_color(COLORS[i % 3].clone().into());

            loop {
                let mut frame = Frame::new();
                // add word text to frame
                frame.add(&word_text);
                // submit frame
                window.submit_frame(frame).await;
            }

            // this is never reached but informs the compiler about the return type
            return Result::<PFutureReturns, anyhow::Error>::Ok(PFutureReturns::NeverReturn);
        };

        // show word screen for 500ms or until either R, G, B is pressed
        let res = word_screen
            .or(sleep(2.0))
            .or(window.wait_for_keypress(vec![Key::R, Key::G, Key::B]))
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
        window.submit_frame(frame).await;
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
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
        window,
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
    spawn_task(experiment(win_handle.clone()));

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
        task::block_on(run(event_loop, window));
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
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
