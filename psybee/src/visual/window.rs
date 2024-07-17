#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_channel::{bounded, Receiver, Sender};
use async_lock::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use derive_debug::Dbg;
use futures_lite::future::block_on;
use futures_lite::Future;
use nalgebra;
use palette::IntoColor;
use pyo3::prelude::*;
use send_wrapper::SendWrapper;

use super::geometry::Size;
use super::stimuli::Stimulus;
use crate::input::{Event, EventHandler, EventHandlerId, EventHandlingExt, EventKind, EventReceiver};
use crate::renderer::{Renderable, Renderer};
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

/// Describes the physical aspect of a window.
#[derive(Debug, Clone, Copy)]
pub struct WindowPhysicalProperties {
    /// The width of the window in pixels.
    pub width: u32,
    /// The height of the window in pixels.
    pub height: u32,
    /// The width of the window in meters.
    pub width_m: f32,
    /// The pixel aspect ratio.
    pub pixel_aspect_ratio: f32,
    /// The viewing distance in meters.
    pub viewing_distance: f32,
}

/// How to block when presenting a frame.
/// A Window represents a window on the screen. It is used to create stimuli and
/// to submit them to the screen for rendering. Each window has a render task
/// that is responsible for rendering stimuli to the screen.
#[derive(Clone, Dbg)]
#[pyclass]
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
    pub physical_properties: WindowPhysicalProperties,
    /// The color format used for rendering.
    pub color_format: ColorFormat,

    /// Vector of stimuli that will be added to each frame automatically.
    #[dbg(placeholder = "...")]
    pub stimuli: Arc<Mutex<Vec<Box<dyn Stimulus>>>>,

    // EVENT HANDLING
    /// Event handlers for the window. Handlers are stored in a HashMap with
    /// the event id as the key.
    #[dbg(placeholder = "...")]
    pub event_handlers: Arc<RwLock<HashMap<EventHandlerId, (EventKind, EventHandler)>>>,

    /// global options
    pub options: Arc<Mutex<crate::options::GlobalOptions>>,
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
        EventReceiver {
            receiver: self.event_broadcast_receiver.activate_cloned(),
        }
    }

    pub fn run_on_render_thread<R, Fut>(&self, task: impl FnOnce() -> Fut + 'static + Send) -> R
    where
        Fut: Future<Output = R> + 'static + Send,
        R: std::marker::Send + 'static,
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
    pub fn present(&self, frame: Frame) {
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
        let mut frame = Frame {
            stimuli: Arc::new(Mutex::new(Vec::new())),
            color_format: self.color_format,
            bg_color: super::color::RawRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };

        // TODO: is this efficient?
        for stimulus in self.stimuli.lock_blocking().iter() {
            frame.add(dyn_clone::clone_box(&**stimulus));
        }

        return frame;
    }
}

#[pyo3::prelude::pymethods]
impl Window {
    #[pyo3(name = "get_frame")]
    fn py_get_frame(&self) -> Frame {
        self.get_frame()
    }

    #[pyo3(name = "present")]
    fn py_present(&self, frame: Frame) {
        self.present(frame);
    }

    /// Add an event handler to the window. The event handler will be called
    /// whenever an event of the specified kind occurs.
    ///
    /// Parameters
    /// ----------
    /// kind : EventKind
    ///   The kind of event to listen for.
    /// callback : callable
    ///  The callback that will be called when the event occurs. The callback should take a single argument, an instance of `Event`.
    #[pyo3(name = "add_event_handler")]
    fn py_add_event_handler(&self, kind: String, callback: Py<PyAny>, py: Python<'_>) {
        let kind = EventKind::from_str(&kind).expect("Invalid event kind");

        let rust_callback_fn = move |event: Event| -> bool {
            Python::with_gil(|py| -> PyResult<()> {
                    callback.call1(py, (event,))
                            .expect("Error calling callback in event handler. Make sure the callback takes a single argument of type Event. Error");
                    Ok(())
                }).unwrap();
            false
        };

        let self_wrapper = SendWrapper::new(self);

        // give up the GIL and
        // run the experiment
        py.allow_threads(move || self_wrapper.add_event_handler(kind.into(), rust_callback_fn));
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
    where
        F: Fn(Event) -> bool + 'static + Send + Sync,
    {
        // find a free id
        let id = loop {
            let id = rand::random::<EventHandlerId>();
            if !self.event_handlers.read_blocking().contains_key(&id) {
                break id;
            }
        };

        // add handler
        self.event_handlers
            .write_blocking()
            .insert(id, (kind, Box::new(handler)));

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

    // on wasm, we register our own requestAnimationFrame callback in a separate task
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

                    let suface_texture: wgpu::SurfaceTexture = window_lock
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = suface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                        format: Some(wgpu::TextureFormat::Bgra8Unorm),
                        ..wgpu::TextureViewDescriptor::default()
                    });
                    let mut encoder = window_lock
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // clear the frame
                    {
                        // clear the frame (once the lifetime annoyance is fixed, this can be
                        // removed only a single render pass is needed using
                        // the LoadOp::Clear option)
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

                    frame
                        .prepare(
                            &window_lock.device,
                            &window_lock.queue,
                            &view,
                            &window_lock.config,
                            &window,
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
        // create the renderer
        let mut renderer = {
            let gpu_state = window.read_gpu_state_blocking();
            Renderer::new(&gpu_state.device)
        };

        loop {
            // wait for frame to be submitted
            let frame = rx.recv().await.unwrap();

            // acquire lock on frame
            let frame = frame.lock_blocking();

            // acquire lock on window
            let window_state = window.read_window_state_blocking();
            let gpu_state = window.read_gpu_state_blocking();

            let device = &gpu_state.device;
            let queue = &gpu_state.queue;
            let config = &window_state.config;

            let suface_texture = window_state
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture");

            let view = suface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                ..wgpu::TextureViewDescriptor::default()
            });

            let mut encoder = gpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                // draw frame to obtain vector of renderables
                let renderables = frame.draw(&window);
                let geoms = renderables
                    .into_iter()
                    .map(|r| r.as_geom().unwrap())
                    .collect::<Vec<_>>();

                // prepare renderables
                let rdata = renderer.prepare(device, queue, config, &geoms);

                // create render pass
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

                renderer.render(rpass, device, &geoms, &rdata);
            }

            let _ = gpu_state.queue.submit(Some(encoder.finish()));

            // present the frame
            suface_texture.present();

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
#[pyclass]
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

    fn draw(&self, window: &Window) -> Vec<Renderable> {
        let stimuli = self.stimuli.lock_blocking();
        let mut renderables = Vec::with_capacity(stimuli.len());
        for stimulus in stimuli.iter() {
            renderables.extend(stimulus.draw(window));
        }
        renderables
    }

    /// Add a renderable to the frame.
    pub fn add(&mut self, stimulus: Box<dyn Stimulus>) -> () {
        self.stimuli.lock_blocking().push(stimulus);
    }

    pub fn add_many<E>(&mut self, stimuli: Vec<E>) -> ()
    where
        E: Into<Box<dyn Stimulus>>,
    {
        for stimulus in stimuli {
            self.add(stimulus.into());
        }
    }
}

#[pymethods]
impl Frame {
    #[pyo3(name = "add")]
    fn py_add(&mut self, stimulus: crate::visual::stimuli::PyStimulus) {
        self.add(stimulus.0)
    }
}
