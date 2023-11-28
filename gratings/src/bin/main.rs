extern crate gratings;


use gratings::visual::Renderable;

use web_time::SystemTime;


use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};



use gratings::visual::gratings::GratingsStimulus;
use gratings::visual::text::TextStimulus;

const PI: f32 = std::f32::consts::PI;

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

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    // vector of renderable stimuli
    let mut gratings = GratingsStimulus::new(&device, &surface, &adapter, 100.0, 0.0);

    let mut title = TextStimulus::new(&device, &queue, "Test 1".to_owned(), swapchain_format);

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
                        device_id: _,
                        input: _,
                        is_synthetic: _,
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

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        gratings.update(&mut device, &queue, &mut encoder, &config);
        title.update(&mut device, &queue, &mut encoder, &config);

        {
            // render the stiuli
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

            gratings.render(&mut device, &mut rpass);
            title.render(&mut device, &mut rpass);
        }

        let now = SystemTime::now();
        let frame_duration = now.duration_since(last_time).unwrap().as_millis();

        queue.submit(Some(encoder.finish()));
        frame.present();

        // print time since last frame
        if frame_duration != 16 {
            println!("Time since last frame: {:?} ms", frame_duration);
        }

        if n_frame % 8 == 0 {
            gratings.params.phase += PI;
        }

        // update frame count
        n_frame += 1;

        last_time = now;
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
