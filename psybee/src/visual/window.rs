use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use std::pin::Pin;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::time::Instant;

use async_channel::{bounded, Receiver, Sender};
use derive_debug::Dbg;
use futures_lite::future::block_on;
use futures_lite::Future;
use nalgebra;
use palette::IntoColor;
use pyo3::prelude::*;
use renderer::vello_backend::VelloRenderer;
use send_wrapper::SendWrapper;
use uuid::Uuid;
use winit::window::WindowId;

use super::color::Rgba;
use super::geometry::Size;
use super::stimuli::{Stimulus, WrappedStimulus};
use crate::input::{Event, EventHandler, EventHandlerId, EventHandlingExt, EventKind, EventReceiver};
use crate::{GPUState, RenderThreadChannelPayload};
use renderer::prelude::*;

/// Internal window state. This is used to store the winit window, the wgpu
/// device, the wgpu queue, etc.
#[derive(Dbg)]
pub struct InternalWindowState {
    /// the winit window
    pub winit_window: Arc<winit::window::Window>,
    /// the wgpu surface
    pub surface: wgpu::Surface<'static>,
    /// the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
    /// the renderer
    #[dbg(placeholder = "...")]
    pub renderer: VelloRenderer,
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
#[derive(Dbg)]
pub struct Window {
    // WINDOW STATE
    /// Window ID
    pub winit_id: winit::window::WindowId,
    /// The window state. This contains the underlying winit window, the wgpu
    /// device, the wgpu queue, etc.
    pub(crate) state: InternalWindowState,
    /// The GPU state
    pub(crate) gpu_state: Arc<Mutex<GPUState>>,
    /// The current mouse position. None if the mouse is not over the
    /// window.
    pub(crate) mouse_position: Option<(Size, Size)>,
    /// Stores if the mouse cursor is currently visible.
    pub(crate) mouse_cursor_visible: bool,

    // CHANNELS FOR COMMUNICATION BETWEEN THREADS
    /// Broadcast sender for keyboard events. Used by the experiment task to
    /// send keyboard events to the main window task.
    pub(crate) event_broadcast_sender: async_broadcast::Sender<Event>,
    /// Broadcast receiver for keyboard events. Used by the main window task to
    /// send keyboard events to the experiment task.
    pub(crate) event_broadcast_receiver: async_broadcast::InactiveReceiver<Event>,
    // PHYSICAL WINDOW PROPERTIES
    /// Physical width of the window in millimeters.
    pub physical_properties: WindowPhysicalProperties,

    /// Vector of stimuli that will be added to each frame automatically.
    #[dbg(placeholder = "...")]
    pub stimuli: Arc<Mutex<Vec<Box<dyn Stimulus>>>>,

    // EVENT HANDLING
    /// Event handlers for the window. Handlers are stored in a HashMap with
    /// the event id as the key.
    #[dbg(placeholder = "...")]
    pub event_handlers: Arc<Mutex<HashMap<EventHandlerId, (EventKind, EventHandler)>>>,

    /// global options
    pub options: Arc<Mutex<crate::options::GlobalOptions>>,
}

impl Window {
    /// Creates a new physical input receiver that will receive physical input
    /// events from the window.
    pub fn create_event_receiver(&self) -> EventReceiver {
        EventReceiver {
            receiver: self.event_broadcast_receiver.activate_cloned(),
        }
    }

    /// Submits a frame to the render task. This will in turn call the prepare()
    /// and render() functions of all renderables in the frame.
    /// This will block until the frame has been consumed by the render task.
    pub fn present(&mut self, frame: &Frame) {
        let win_state = &mut self.state;
        let gpu_state = &mut self.gpu_state.lock().unwrap();

        let device = &gpu_state.device;
        let queue = &gpu_state.queue;
        let config = &win_state.config;

        let suface_texture = win_state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let width = suface_texture.texture.size().width;
        let height = suface_texture.texture.size().height;

        let view = suface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            ..wgpu::TextureViewDescriptor::default()
        });

        &win_state
            .renderer
            .render_to_surface2(device, queue, &suface_texture, &frame.scene);

        // present the frame
        suface_texture.present();
    }

    pub fn close(&self) {
        todo!()
    }

    /// Set the visibility of the mouse cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        let window = &self.state.winit_window;
        window.set_cursor_visible(false);
        self.mouse_cursor_visible = visible;
    }

    /// Returns true if the mouse cursor is currently visible.
    pub fn cursor_visible(&self) -> bool {
        return self.mouse_cursor_visible;
    }

    /// Returns the mouse position. None if cursor not in window.
    pub fn mouse_position(&self) -> Option<(Size, Size)> {
        return self.mouse_position.clone();
    }

    /// Returns the 4x4 matrix than when applied to pixel coordinates will transform
    /// them to normalized device coordinates. Pixel coordinates are in a
    /// coordinate system with (0.0,0.0) in the center of the screen and
    /// (half of screen width in px, half of screen height in px) in the top right
    /// corner of the screen.
    #[rustfmt::skip]
    pub fn transformation_matrix_to_ndc(width_px: u32, height_px: u32) -> nalgebra::Matrix3<f64> {
        nalgebra::Matrix3::new(
            2.0 / width_px as f64,0.0, 0.0,
            0.0, 2.0 / height_px as f64, 0.0,
            0.0, 0.0, 1.0,
        )
    }

    /// Returns the size of the window in pixels.
    pub fn size(&self) -> (u32, u32) {
        let props = self.physical_properties;
        (props.width, props.height)
    }
}

#[pyclass(name = "Window")]
#[derive(Debug, Clone)]
pub struct WrappedWindow(pub(crate) Arc<Mutex<Window>>);

impl WrappedWindow {
    pub fn new(window: Window) -> Self {
        Self(Arc::new(Mutex::new(window)))
    }

    pub fn winit_id(&self) -> WindowId {
        self.inner().winit_id
    }

    pub fn inner(&self) -> MutexGuard<'_, Window> {
        self.0.lock().unwrap()
    }

    pub fn create_event_receiver(&self) -> EventReceiver {
        self.inner().create_event_receiver()
    }

    pub fn present(&self, frame: &Frame) {
        self.inner().present(frame)
    }

    pub fn close(&self) {
        self.inner().close()
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.inner().set_cursor_visible(visible)
    }

    pub fn cursor_visible(&self) -> bool {
        self.inner().cursor_visible()
    }

    pub fn mouse_position(&self) -> Option<(Size, Size)> {
        self.inner().mouse_position()
    }

    pub fn transformation_matrix_to_ndc(width_px: u32, height_px: u32) -> nalgebra::Matrix3<f64> {
        Window::transformation_matrix_to_ndc(width_px, height_px)
    }

    pub fn size(&self) -> (u32, u32) {
        self.inner().size()
    }

    // Create a new frame with a black background.
    pub fn get_frame(&self) -> Frame {
        // check if self.state is locked
        let props = &self.inner().physical_properties;

        let width = props.width;
        let height = props.height;

        let bg = super::color::Rgba {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        // create new scene
        let mut scene = Scene::new(bg.into(), width, height);
        let mut frame = Frame {
            bg_color: bg,
            scene,
            window: self.clone(),
        };

        return frame;
    }
}

#[pymethods]
impl WrappedWindow {
    #[pyo3(name = "get_frame")]
    fn py_get_frame(&self, py: Python) -> Frame {
        let f = self.get_frame();
        f
    }

    #[pyo3(name = "present")]
    fn py_present(&self, frame: &Frame, py: Python) {
        self.present(frame);
    }

    #[getter(cursor_visible)]
    fn py_cursor_visible(&self) -> bool {
        self.cursor_visible()
    }

    #[setter(cursor_visible)]
    fn py_set_cursor_visible(&self, visible: bool) {
        self.set_cursor_visible(visible);
    }

    #[pyo3(name = "get_size")]
    fn py_get_size(&self, py: Python) -> (u32, u32) {
        self.size()
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

        py.allow_threads(move || self_wrapper.add_event_handler(kind.into(), rust_callback_fn));
    }
}

impl EventHandlingExt for WrappedWindow {
    fn remove_event_handler(&self, id: EventHandlerId) {
        self.inner().event_handlers.lock().unwrap().remove(&id);
    }

    fn dispatch_event(&self, event: Event) -> bool {
        let mut handled = false;

        let event_handlers = {
            let c = self.inner().event_handlers.clone();
            c
        };
        for (id, (kind, handler)) in event_handlers.lock().unwrap().iter() {
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
            if !self.inner().event_handlers.lock().unwrap().contains_key(&id) {
                break id;
            }
        };

        // add handler
        self.inner()
            .event_handlers
            .lock()
            .unwrap()
            .insert(id, (kind, Box::new(handler)));

        return id;
    }
}

#[derive(Dbg)]
#[pyclass]
#[pyo3(unsendable)]
pub struct Frame {
    pub bg_color: super::color::Rgba,
    #[dbg(placeholder = "...")]
    scene: VelloScene,
    /// The window that the frame is associated with.
    window: WrappedWindow,
}

impl Frame {
    /// Set the background color of the frame.
    pub fn set_bg_color(&mut self, bg_color: Rgba) {
        self.bg_color = bg_color;
    }

    /// Draw onto the frame.
    pub fn draw(&mut self, stimulus: WrappedStimulus) {
        let now = Instant::now();
        let mut stimulus = stimulus.lock().unwrap();
        stimulus.update_animations(now, &self.window.inner());
        stimulus.draw(&mut self.scene, &self.window.inner());
    }
}

#[pymethods]
impl Frame {
    #[pyo3(name = "draw")]
    fn py_draw(&mut self, stimulus: crate::visual::stimuli::PyStimulus) {
        self.draw(stimulus.0);
    }

    #[getter(bg_color)]
    fn py_get_bg_color(&self) -> super::color::Rgba {
        self.bg_color
    }

    #[setter(bg_color)]
    fn py_set_bg_color(&mut self, bg_color: super::color::Rgba) {
        self.bg_color = bg_color;
    }
}
