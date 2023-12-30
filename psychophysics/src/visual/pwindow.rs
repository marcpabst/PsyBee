#[cfg(target_arch = "wasm32")]
use crate::request_animation_frame;
use crate::utils::BlockingLock;
use crate::{async_sleep, input::Key, PFutureReturns};
use async_lock::{Mutex, MutexGuard};
use futures_lite::Future;
use palette::IntoColor;

use crate::visual::color::ColorFormat;
use async_trait::async_trait;

use atomic_float::AtomicF64;

use nalgebra;

use async_channel::{bounded, Receiver, Sender};
use futures_lite::future::block_on;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::pin::Pin;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};
#[cfg(target_arch = "wasm32")]
use web_sys::console;

use super::Renderable;

/// This is the main window struct. It contains all the information needed to render stimuli and control the graphics pipeline.
/// Since the window might be shared across threads, you will usually have access to it through a WindowHandle.
pub struct WindowState {
    // the winit window
    pub window: winit::window::Window,
    // the event loop proxy
    pub event_loop_proxy: winit::event_loop::EventLoopProxy<()>,
    // the wgpu device
    pub device: wgpu::Device,
    // the wgpu instance
    pub instance: wgpu::Instance,
    // the wgpu adapter
    pub adapter: wgpu::Adapter,
    // the wgpu queue
    pub queue: wgpu::Queue,
    // the wgpu surface
    pub surface: wgpu::Surface,
    // the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
}

/// A Winod is a shared reference to a PWindow. It is the main way to interact with the window.
/// It also stores the channels used for communication between the main task, the render task and the experiment task.
#[derive(Clone)]
pub struct Window {
    /// The window state.
    pub state: Arc<Mutex<WindowState>>,
    /// Broadcast receiver for keyboard events. Used by the main window task to send keyboard events to the experiment task.
    pub keyboard_receiver: async_broadcast::InactiveReceiver<winit::event::KeyboardInput>,
    /// Channel for frame submission. Used by the experiment task to submit frames to the render task.
    pub frame_sender: Sender<Arc<Mutex<Frame>>>,
    /// Channel for frame submission. Used by the experiment task to submit frames to the render task.
    pub frame_receiver: Receiver<Arc<Mutex<Frame>>>,
    /// Channel for frame consumption. Used by the render task to notify the experiment task that a frame has been consumed.
    pub frame_ok_sender: Sender<bool>,
    /// Channel for frame consumption. Used by the render task to notify the experiment task that a frame has been consumed.
    pub frame_ok_receiver: Receiver<bool>,
    /// Channel for sending a future to the render task. The future will be executed on the render thread.
    pub render_task_sender:
        Sender<Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()>>>>>,
    /// Channel for receiving functions that should be executed on the render thread.
    pub render_task_receiver:
        Receiver<Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()>>>>>,
    /// Physical width of the window in millimeters.
    pub physical_width: Arc<AtomicF64>,
    /// Viewing distance in millimeters.
    pub viewing_distance: Arc<AtomicF64>,
    /// The color format used for rendering.
    pub color_format: ColorFormat,
}

// mark Window as Send and Sync
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Window {
    /// Returns a MutexGuard to the PWindow behind the mutex.
    pub fn get_window_state_blocking(&self) -> MutexGuard<WindowState> {
        return self.state.lock_blocking();
    }

    pub async fn get_window_state(&self) -> MutexGuard<WindowState> {
        return self.state.lock().await;
    }

    /// Listens for the specified keypresses and returns the key that was pressed and the time it took to press it.
    /// When a keypress is detected, the Future returns a PFutureReturns::KeyPress.
    pub async fn wait_for_keypress<T, I>(
        &self,
        keys: T,
    ) -> Result<PFutureReturns, anyhow::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Key>,
    {
        let start: web_time::Instant = web_time::Instant::now();

        let mut keyboard_receiver = self.keyboard_receiver.activate_cloned();

        let key_vec: Vec<Key> = keys.into_iter().map(|k| k.into()).collect();

        let kc: winit::event::VirtualKeyCode;
        loop {
            // wait for buttons pres
            let e = keyboard_receiver.recv().await.map_err(|_| {
                anyhow::anyhow!("Failed to receive keypress from channel")
            })?;

            // check if keypress matches any of the keys
            if key_vec.contains(&e.virtual_keycode.unwrap().into()) || key_vec.is_empty()
            {
                kc = e.virtual_keycode.unwrap();
                break;
            }
        }

        return Ok(PFutureReturns::KeyPress((
            kc.into(),
            web_time::Instant::now().duration_since(start),
        )));
    }

    /// Send a task to the render thread. The task will be executed on the render thread and will block the current thread until it is finished.
    pub fn run_on_render_thread<R, Fut>(&self, task: impl FnOnce() -> Fut + 'static) -> R
    where
        Fut: Future<Output = R> + 'static,
        R: std::marker::Send + 'static,
    {
        // create channel to receive result
        let (tx, rx) = bounded(1);

        // create task
        let rtask = move || {
            let task = async move {
                let result = task().await;
                block_on(tx.send(result));
            };
            Box::pin(task) as Pin<Box<dyn Future<Output = ()>>>
        };

        let rtask_boxed =
            Box::new(rtask) as Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()>>>>;

        // send task
        block_on(self.render_task_sender.send(rtask_boxed));

        // wait for result
        return block_on(rx.recv()).unwrap();
    }

    /// Same as wait_for_keypress, but waits for any keypress.
    pub async fn wait_for_any_keypress(&self) -> Result<PFutureReturns, anyhow::Error> {
        let empty_vec: Vec<Key> = Vec::new();
        return self.wait_for_keypress(empty_vec).await;
    }

    /// Submits a frame to the render task. This will in turn call the prepare() and render() functions of all renderables in the frame.
    /// The future will return when the frame has been commited to the global render queue.
    pub fn submit_frame(&self, frame: Frame) {
        let frame_sender = self.frame_sender.clone();
        let frame_ok_receiver = self.frame_ok_receiver.clone();

        // submit frame to channel
        block_on(frame_sender.send(Arc::new(Mutex::new(frame))));

        // wait for frame to be consumed
        block_on(frame_ok_receiver.recv());
    }

    pub fn close(&self) {
        self.state.lock_blocking().event_loop_proxy.send_event(());
    }

    /// Returns the color format.
    pub fn get_color_format(&self) -> ColorFormat {
        self.color_format
    }

    /// Returns the 4x4 matrix than when applied to pixel coordinates will transform them to normalized device coordinates.
    /// Pixel coordinates are in a coordinate system with (0.0,0.0) in the center of the screen and
    /// (half of screen width in px, half of screen height in px) in the top right corner of the screen.
    #[rustfmt::skip]
    pub fn transformation_matrix_to_ndc(width_px: u32, height_px: u32) -> nalgebra::Matrix4<f64> {
        // TODO: this could be cached to avoid locking the mutex

        nalgebra::Matrix4::new(
            2.0 / width_px as f64,0.0, 0.0, 0.0,
            0.0, 2.0 / height_px as f64, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0,0.0, 0.0, 1.0,
        )
    }

    // Create a new frame with a black background.
    pub fn get_frame(&self) -> Frame {
        Frame {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color: wgpu::Color::BLACK,
        }
    }

    // Create a new frame with a custom background color.
    pub fn get_frame_with_bg_color(
        &self,
        bg_color: impl IntoColor<palette::Xyza<palette::white_point::D65, f32>>,
    ) -> Frame {
        let bg_color = self.color_format.convert_to_raw_rgba(bg_color);

        Frame {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color: (bg_color).into(),
        }
    }
}

/// This is the second render task. It is used to execute tasks on the render thread when running on WASM.
pub async fn render_task2(window_handle: Window) {
    // loop
    loop {
        // wait until task is available
        let task = window_handle.render_task_receiver.recv().await.unwrap();
        // await the task
        task().await;
    }
}

/// This is the window's main render task. On native, it will submit frames when they are ready (and block when an approriate presentation mode is used).
/// On wasm, it will submit frames when the browser requests a new frame.
pub async fn render_task(window_handle: Window) {
    // get rx and tx from handle
    let tx = window_handle.frame_ok_sender.clone();
    let rx = window_handle.frame_receiver.clone();

    // on wasm, we register our own requestAnimationFrame callback in a separate task
    #[cfg(target_arch = "wasm32")]
    {
        log::debug!(
            "Render task running on thread {:?}",
            std::thread::current().id()
        );

        // here, we create a closure that will be called by requestAnimationFrame
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());

            let tx = tx.clone();
            let rx = rx.clone();
            let window_handle = window_handle.clone();

            let async_task = async move {
                // check if there is a frame available
                let try_frame = rx.try_recv();

                if try_frame.is_ok() {
                    let frame = try_frame.unwrap();
                    // acquire lock on frame
                    let mut frame = frame.lock_blocking();

                    // acquire lock on window
                    let window_lock = window_handle.get_window_state_blocking();

                    let suface_texture: wgpu::SurfaceTexture = window_lock
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = suface_texture.texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            format: Some(wgpu::TextureFormat::Bgra8Unorm),
                            ..wgpu::TextureViewDescriptor::default()
                        },
                    );
                    let mut encoder = window_lock.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: None },
                    );

                    // clear the frame
                    {
                        // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                        // using the LoadOp::Clear option)
                        let rpass =
                            &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(
                                    wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(
                                                frame.bg_color.into(),
                                            ),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    },
                                )],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                    }

                    frame
                        .prepare(
                            &window_lock.device,
                            &window_lock.queue,
                            &view,
                            &window_lock.config,
                            &window_handle,
                        )
                        .await;

                    frame.render(&mut encoder, &view);

                    window_lock.queue.submit(Some(encoder.finish()));
                    suface_texture.present();

                    // notify sender that frame has been consumed
                    let _ = tx.try_send(true);
                };
            };

            // spawn the async task
            wasm_bindgen_futures::spawn_local(async_task);
        }));
        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    // on native, we submit frames when they are ready
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::debug!(
            "Render task running on thread {:?}",
            std::thread::current().id()
        );

        loop {
            // wait for frame to be submitted
            let frame = rx.recv().await.unwrap();

            // acquire lock on frame
            let mut frame = (frame.lock_blocking());

            // acquire lock on window
            let window_lock = window_handle.state.lock_blocking();

            let suface_texture = window_lock
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture");
            let view = suface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(wgpu::TextureFormat::Bgra8Unorm),
                    ..wgpu::TextureViewDescriptor::default()
                });
            let mut encoder = window_lock
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            // clear the frame
            {
                log::info!("Clearing frame with color {:?}", frame.bg_color);
                // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                // using the LoadOp::Clear option)
                let _rpass =
                    &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            frame
                .prepare(
                    &window_lock.device,
                    &window_lock.queue,
                    &view,
                    &window_lock.config,
                    &window_handle,
                )
                .await;
            frame.render(&mut encoder, &view);

            window_lock.queue.submit(Some(encoder.finish()));
            suface_texture.present();

            // notify sender that frame has been consumed
            let _ = block_on(tx.send(true));
        }
    }
}
/// A frame is a collection of renderables that will be rendered together.
/// Rendering is lazy, i.e. the prepare() and render() functions of the renderables
/// will only be called once the frame is submitted to the render task.
pub struct Frame {
    renderables: Arc<Mutex<Vec<Box<dyn Renderable>>>>,
    pub bg_color: wgpu::Color,
}

#[async_trait(?Send)]
impl Renderable for Frame {
    async fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
        window_handle: &Window,
    ) -> () {
        // call prepare() on all renderables
        for renderable in &mut self.renderables.lock().await.iter_mut() {
            renderable
                .prepare(device, queue, view, config, window_handle)
                .await;
        }
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // call render() on all renderables
        for renderable in &mut self.renderables.lock_blocking().iter_mut() {
            renderable.render(enc, view);
        }
    }
}

impl Frame {
    /// Add a renderable to the frame.
    pub fn add(&mut self, renderable: &(impl Renderable + Clone + 'static)) -> () {
        let renderable = Box::new(renderable.clone());
        self.renderables.lock_blocking().push(renderable);
    }
}

// mark Frame as Send and Sync
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}
