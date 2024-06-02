#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use async_channel::{bounded, Receiver, Sender};
use async_lock::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use atomic_float::AtomicF64;
use derive_debug::Dbg;
use futures_lite::future::block_on;
use futures_lite::Future;
use nalgebra;
use palette::IntoColor;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;

use super::geometry::Size;
use super::stimuli::Stimulus;
use crate::input::{Event, EventHandler, EventHandlerId, EventHandlingExt, EventKind, EventReceiver};
#[cfg(target_arch = "wasm32")]
use crate::request_animation_frame;
use crate::visual::color::ColorFormat;
use crate::{GPUState, RenderThreadChannelPayload};

/// Internal window state. This is used to store the winit window, the wgpu
/// device, the wgpu queue, etc.
#[derive(Debug)]
pub struct InternalWindowState {
    // the winit window
    pub window: Arc<winit::window::Window>,
    // the wgpu surface
    pub surface: wgpu::Surface<'static>,
    // the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
}

/// How to block when presenting a frame.
pub enum BlockingStrategy {
    /// Will render the current frame using a command buffer and submit it to the GPU, then immediately return.
    /// Note that this may still block depending on the maximum number of frames in flight, i.e. if you submit
    /// too many frames in short succession, this will block until the GPU has caught up with the work.
    DoNotBlock,

    // /// Will block until the previous the GPU has finished processing the previous frame. Depending on
    // /// the platform, this may be identical to `BlockUntilVBlank`, but there is generally no guarantee
    // /// that this will return in sync with the vertical blanking interval or the actual frame presentation.
    // BlockUntilPreviousFrameFinished,
    /// Will block until the vertical blanking interval at which the current frame will be presented,
    /// then return as quickly as possible.
    BlockUntilVBlank,

    /// Will block until the vertical blanking interval at which the current frame will be presented,
    /// then return as quickly as possible. This will also verify that the frame has been presented and
    /// that no frame has been dropped. This is not supported on all platforms.
    BlockUntilBVBlankVerified,
}

/// A Window represents a window on the screen. It is used to create stimuli and
/// to submit them to the screen for rendering. Each window has a render task
/// that is responsible for rendering stimuli to the screen.
#[derive(Clone, Dbg)]
pub struct Window {
    // WINDOW STATE
    /// The window state. This contains the underlying winit window, the wgpu
    /// device, the wgpu queue, etc.
    pub(crate) state: Arc<RwLock<InternalWindowState>>,
    /// The GPU state
    pub(crate) gpu_state: Arc<RwLock<GPUState>>,
    /// The current mouse position. None if the mouse is not over the
    /// window.
    pub(crate) mouse_position: Arc<Mutex<Option<(Size, Size)>>>,
    /// Stores if the mouse cursor is currently visible.
    pub(crate) mouse_cursor_visible: Arc<AtomicBool>,

    // CHANNELS FOR COMMUNICATION BETWEEN THREADS
    /// Broadcast sender for keyboard events. Used by the experiment task to
    /// send keyboard events to the main window task.
    pub(crate) event_broadcast_sender: async_broadcast::Sender<Event>,
    /// Broadcast receiver for keyboard events. Used by the main window task to
    /// send keyboard events to the experiment task.
    pub(crate) event_broadcast_receiver: async_broadcast::InactiveReceiver<Event>,
    /// Channel for frame submission. Used by the experiment task to submit
    /// frames to the render task.
    pub(crate) frame_channel_sender: Sender<Arc<Mutex<Frame>>>,
    /// Channel for frame submission. Used by the experiment task to submit
    /// frames to the render task.
    pub(crate) frame_channel_receiver: Receiver<Arc<Mutex<Frame>>>,
    /// Channel for frame consumption. Used by the render task to notify the
    /// experiment task that a frame has been consumed.
    pub(crate) frame_consumed_channel_sender: Sender<bool>,
    /// Channel for frame consumption. Used by the render task to notify the
    /// experiment task that a frame has been consumed.
    pub(crate) frame_consumed_channel_receiver: Receiver<bool>,
    /// render_task_sender
    pub(crate) render_task_sender: Sender<RenderThreadChannelPayload>,

    // PHYSICAL WINDOW PROPERTIES
    /// Physical width of the window in millimeters.
    pub physical_width: Arc<AtomicF64>,
    /// Viewing distance in millimeters.
    pub viewing_distance: Arc<AtomicF64>,
    /// The color format used for rendering.
    pub color_format: ColorFormat,
    /// The window's width in pixels.
    pub width_px: Arc<AtomicU32>,
    /// The window's height in pixels.
    pub height_px: Arc<AtomicU32>,

    /// Vector of stimuli that will be added to each frame automatically.
    #[dbg(placeholder = "...")]
    pub stimuli: Arc<Mutex<Vec<Box<dyn Stimulus>>>>,

    // EVENT HANDLING
    /// Event handlers for the window. Handlers are stored in a HashMap with
    /// the event id as the key.
    #[dbg(placeholder = "...")]
    pub event_handlers: Arc<RwLock<HashMap<EventHandlerId, (EventKind, EventHandler)>>>,
}

impl Window {
    /// Returns a MutexGuard to the WindowState behind the mutex.
    pub fn read_window_state_blocking(&self) -> RwLockReadGuard<InternalWindowState> {
        return self.state.read_blocking();
    }

    pub fn write_window_state_blocking(&self) -> RwLockWriteGuard<InternalWindowState> {
        return self.state.write_blocking();
    }

    /// Returns a MutexGuard to the WindowState behind the mutex.
    pub fn read_gpu_state_blocking(&self) -> RwLockReadGuard<GPUState> {
        return self.gpu_state.read_blocking();
    }

    pub fn write_gpu_state_blocking(&self) -> RwLockWriteGuard<GPUState> {
        return self.gpu_state.write_blocking();
    }

    /// Creates a new physical input receiver that will receive physical input
    /// events from the window.
    pub fn create_event_receiver(&self) -> EventReceiver {
        EventReceiver { receiver: self.event_broadcast_receiver.activate_cloned() }
    }

    pub fn run_on_render_thread<R, Fut>(&self, task: impl FnOnce() -> Fut + 'static + Send) -> R
        where Fut: Future<Output = R> + 'static + Send,
              R: std::marker::Send + 'static
    {
        // create channel to receive result
        let (tx, rx) = bounded(1);

        // create task
        let rtask = move || {
            let task = async move {
                let result = task().await;
                block_on(tx.send(result)).expect("Failed to send result");
            };

            Box::pin(task) as Pin<Box<dyn Future<Output = ()> + Send>>
        };

        let rtask_boxed = Box::new(rtask) as Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

        log::info!("Sending task to render task");

        // send task
        block_on(self.render_task_sender.send(rtask_boxed)).expect("Failed to send task to render task");

        log::info!("Waiting for result");

        // wait for result
        let ret = block_on(rx.recv()).unwrap();

        log::info!("Got result");

        return ret;
    }

    /// Submits a frame to the render task. This will in turn call the prepare()
    /// and render() functions of all renderables in the frame.
    /// This will block until the frame has been consumed by the render task.
    pub fn present(&self, frame: Frame, blocking_strategy: Option<BlockingStrategy>) {
        let frame_sender = self.frame_channel_sender.clone();
        let frame_ok_receiver = self.frame_consumed_channel_receiver.clone();

        // submit frame to channel
        block_on(frame_sender.send(Arc::new(Mutex::new(frame)))).expect("Failed to send frame");

        // wait for frame to be consumed
        block_on(frame_ok_receiver.recv()).expect("Failed to receive frame_ok");
    }

    pub fn close(&self) {
        todo!()
    }

    /// Returns the color format.
    #[deprecated(note = "Color format handling will change in the future.")]
    pub fn get_color_format(&self) -> ColorFormat {
        self.color_format
    }

    /// Set the visibility of the mouse cursor.
    pub fn set_cursor_visible(&self, visible: bool) {
        self.state.read_blocking().window.set_cursor_visible(visible);
        self.mouse_cursor_visible.store(visible, Ordering::Relaxed);
    }

    /// Returns true if the mouse cursor is currently visible.
    pub fn cursor_visible(&self) -> bool {
        self.mouse_cursor_visible.load(Ordering::Relaxed)
    }

    /// Returns the mouse position. None if cursor not in window.
    pub fn mouse_position(&self) -> Option<(Size, Size)> {
        self.mouse_position.lock_blocking().clone()
    }

    /// Returns the 4x4 matrix than when applied to pixel coordinates will transform
    /// them to normalized device coordinates. Pixel coordinates are in a
    /// coordinate system with (0.0,0.0) in the center of the screen and
    /// (half of screen width in px, half of screen height in px) in the top right
    /// corner of the screen.
    #[rustfmt::skip]
    pub fn transformation_matrix_to_ndc(width_px: u32, height_px: u32) -> nalgebra::Matrix3<f64> {
        // TODO: this could be cached to avoid locking the mutex

        nalgebra::Matrix3::new(
            2.0 / width_px as f64,0.0, 0.0,
            0.0, 2.0 / height_px as f64, 0.0,
            0.0, 0.0, 1.0,
        )

    }

    // Create a new frame with a black background.
    pub fn get_frame(&self) -> Frame {
        let mut frame = Frame { stimuli: Arc::new(Mutex::new(Vec::new())),
                                color_format: self.color_format,
                                bg_color: super::color::RawRgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } };

        // TODO: is this efficient?
        for stimulus in self.stimuli.lock_blocking().iter() {
            frame.add(dyn_clone::clone_box(&**stimulus));
        }

        return frame;
    }

    /// Returns the physical width of the window in millimeters.
    pub fn physical_width(&self) -> f64 {
        self.physical_width.load(Ordering::Relaxed)
    }

    /// Sets the physical width of the window in millimeters.
    pub fn set_physical_width(&self, width: f64) {
        self.physical_width.store(width, Ordering::Relaxed);
    }

    /// Returns the viewing distance in millimeters.
    pub fn viewing_distance(&self) -> f64 {
        self.viewing_distance.load(Ordering::Relaxed)
    }

    /// Sets the viewing distance in millimeters.
    pub fn set_viewing_distance(&self, distance: f64) {
        self.viewing_distance.store(distance, Ordering::Relaxed);
    }

    /// Returns the width of the window in pixels.
    pub fn width_px(&self) -> u32 {
        self.width_px.load(Ordering::Relaxed)
    }

    /// Returns the height of the window in pixels.
    pub fn height_px(&self) -> u32 {
        self.height_px.load(Ordering::Relaxed)
    }
}

impl EventHandlingExt for Window {
    fn remove_event_handler(&self, id: EventHandlerId) {
        self.event_handlers.write_blocking().remove(&id);
    }

    fn dispatch_event(&self, event: Event) -> bool {
        let mut handled = false;

        for (id, (kind, handler)) in self.event_handlers.read_blocking().iter() {
            if kind == &event.kind() {
                handled |= handler(event.clone());
            }
        }

        return handled;
    }

    fn add_event_handler<F>(&self, kind: EventKind, handler: F) -> EventHandlerId
        where F: Fn(Event) -> bool + 'static + Send + Sync
    {
        // find a free id
        let id = loop {
            let id = rand::random::<EventHandlerId>();
            if !self.event_handlers.read_blocking().contains_key(&id) {
                break id;
            }
        };

        // add handler
        self.event_handlers.write_blocking().insert(id, (kind, Box::new(handler)));

        return id;
    }
}

/// This is the window's main render task. On native, it will submit frames when
/// they are ready (and block when an approriate presentation mode is used).
/// On wasm, it will submit frames when the browser requests a new frame.
pub async fn render_task(window: Window) {
    // get rx and tx from handle
    let tx = window.frame_consumed_channel_sender.clone();
    let rx = window.frame_channel_receiver.clone();

    // on wasm, we register our own requestAnimationFrame callback in a separate
    // task
    #[cfg(target_arch = "wasm32")]
    {
        log::debug!("Render task running on thread {:?}", std::thread::current().id());

        // here, we create a closure that will be called by requestAnimationFrame
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());

            let tx = tx.clone();
            let rx = rx.clone();
            let window_handle = window.clone();

            let async_task = async move {
                // check if there is a frame available
                let try_frame = rx.try_recv();

                if try_frame.is_ok() {
                    let frame = try_frame.unwrap();
                    // acquire lock on frame
                    let mut frame = frame.lock_blocking();

                    // acquire lock on window
                    let window_lock = window.get_window_state_blocking();

                    let suface_texture: wgpu::SurfaceTexture = window_lock.surface.get_current_texture().expect("Failed to acquire next swap chain texture");
                    let view = suface_texture.texture
                                             .create_view(&wgpu::TextureViewDescriptor { format: Some(wgpu::TextureFormat::Bgra8Unorm),
                                                                                         ..wgpu::TextureViewDescriptor::default() });
                    let mut encoder = window_lock.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // clear the frame
                    {
                        // clear the frame (once the lifetime annoyance is fixed, this can be
                        // removed only a single render pass is needed using
                        // the LoadOp::Clear option)
                        let _rpass =
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

                    frame.prepare(&window_lock.device, &window_lock.queue, &view, &window_lock.config, &window).await;

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

        let flip_count = Arc::new(AtomicU32::new(0));
        loop {
            // wait for frame to be submitted
            let frame = rx.recv().await.unwrap();

            // acquire lock on frame
            let mut frame = frame.lock_blocking();

            // acquire lock on window
            let window_state = window.read_window_state_blocking();
            let gpu_state = window.read_gpu_state_blocking();

            let suface_texture = window_state.surface.get_current_texture().expect("Failed to acquire next swap chain texture");

            let view = suface_texture.texture
                                     .create_view(&wgpu::TextureViewDescriptor { format: Some(wgpu::TextureFormat::Bgra8Unorm),
                                                                                 ..wgpu::TextureViewDescriptor::default() });

            let mut encoder = gpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            // start timer

            // clear the frame
            {
                // clear the frame (once the lifetime annoyance is fixed, this can be removed
                // only a single render pass is needed using the LoadOp::Clear
                // option)
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

            let t_start = std::time::Instant::now();
            frame.prepare(&window, &window_state, &gpu_state).await;
            log::warn!("Frame - Time to prepare: {:?}", t_start.elapsed());

            let t_start = std::time::Instant::now();
            frame.render(&mut encoder, &view);
            log::warn!("Frame - Time to render: {:?}", t_start.elapsed());

            let t_start = std::time::Instant::now();
            let _ = gpu_state.queue.submit(Some(encoder.finish()));
            log::warn!("Frame - Time to submit: {:?}", t_start.elapsed());

            suface_texture.present();

            // drop window_state
            drop(window_state);

            let mut window_state = window.write_window_state_blocking();

       

            #[cfg(target_os = "windows")]
            {
                let hal_surface_callback = |sf: Option<&wgpu::hal::dx12::Surface>| {
                    let dxgi_surface = sf.unwrap();
                    let swap_chain = dxgi_surface.raw_swap_chain().unwrap();

                    // get frame statistics
                    let mut stats = winapi::shared::dxgi::DXGI_FRAME_STATISTICS::default();
                    unsafe { swap_chain.GetFrameStatistics(&mut stats) };

                    // get frame statistics
                    let diff = stats.SyncRefreshCount - flip_count.load(Ordering::Relaxed);
                    flip_count.store(stats.SyncRefreshCount, Ordering::Relaxed);

                    log::warn!("Flips since last frame: {}", diff);
                };

                unsafe { &window_state.surface.as_hal::<wgpu::core::api::Dx12, _, _>(hal_surface_callback) }.unwrap();
            }

            // notify sender that frame has been consumed
            let _ = block_on(tx.send(true));
        }
    }
}

/// A frame is a collection of renderables that will be rendered together.
/// Rendering is lazy, i.e. the prepare() and render() functions of the
/// renderables will only be called once the frame is submitted to the render
/// task.
#[derive(Clone, Dbg)]
pub struct Frame {
    #[dbg(placeholder = "...")]
    pub stimuli: Arc<Mutex<Vec<Box<dyn Stimulus>>>>,
    color_format: ColorFormat,
    pub bg_color: super::color::RawRgba,
}

impl Frame {
    /// Set the background color of the frame.
    pub fn set_bg_color(&mut self, bg_color: impl IntoColor<palette::Xyza<palette::white_point::D65, f32>>) {
        self.bg_color = self.color_format.convert_to_raw_rgba(bg_color);
    }
}

impl Frame {
    async fn prepare(&mut self, window: &Window, window_state: &InternalWindowState, gpu_state: &GPUState) -> () {
        // call prepare() on all renderables

        for renderable in &mut self.stimuli.lock().await.iter_mut() {
            renderable.prepare(window, window_state, gpu_state);
        }
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // call render() on all renderables
        let t_start = std::time::Instant::now();
        let lb = &mut self.stimuli.lock_blocking();

        for renderable in lb.iter_mut() {
            renderable.render(enc, view);
        }
        //log::info!("Time to render stimuli: {:?}", t_start.elapsed());
    }
}

impl Frame {
    /// Add a renderable to the frame.
    pub fn add(&mut self, stimulus: Box<dyn Stimulus>) -> () {
        self.stimuli.lock_blocking().push(stimulus);
    }

    pub fn add_many<E>(&mut self, stimuli: Vec<E>) -> ()
        where E: Into<Box<dyn Stimulus>>
    {
        for stimulus in stimuli {
            self.add(stimulus.into());
        }
    }
}
