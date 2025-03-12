use std::{
    collections::HashMap,
    ops::Deref,
    pin::Pin,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard, RwLock,
    },
    time::Instant,
};

use async_channel::{bounded, Receiver, Sender};
use derive_debug::Dbg;
use futures_lite::{future::block_on, Future};
use nalgebra;
use palette::IntoColor;
use pyo3::prelude::*;
use renderer::{renderer::RendererFactory, wgpu_renderer::WgpuRenderer, DynamicRenderer, DynamicScene};
use send_wrapper::SendWrapper;
use uuid::Uuid;
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::WindowId};

use super::{
    color::LinRgba,
    geometry::Size,
    stimuli::{DynamicStimulus, Stimulus},
};
use crate::{
    app::GPUState,
    input::{Event, EventHandler, EventHandlerId, EventHandlingExt, EventKind, EventReceiver},
    RenderThreadChannelPayload,
};

#[derive(Debug, Clone, Copy)]
pub struct PhysicalScreen {
    /// Pixel/mm of the screen.
    pub pixel_density: f32,
    /// Viewing distance in meters.
    pub viewing_distance: f32,
}

impl PhysicalScreen {
    /// Creates a new physical screen given width in pixels and millimeters.
    pub fn new(width_px: u32, width_mm: f32, viewing_distance: f32) -> Self {
        let pixel_density = width_px as f32 / width_mm;
        Self {
            pixel_density,
            viewing_distance,
        }
    }

    /// Returns the size of the screen in millimeters.
    pub fn size(&self, width_px: u32, height_px: u32) -> (f32, f32) {
        let width_mm = width_px as f32 / self.pixel_density;
        let height_mm = height_px as f32 / self.pixel_density;
        (width_mm, height_mm)
    }

    /// Returns the width of the screen in millimeters.
    pub fn width(&self, width_px: u32) -> f32 {
        width_px as f32 / self.pixel_density
    }

    /// Returns the height of the screen in millimeters.
    pub fn height(&self, height_px: u32) -> f32 {
        height_px as f32 / self.pixel_density
    }

    /// Sets the pixel density of the screen based on the width of the screen in pixels and millimeters.
    pub fn set_pixel_density(&mut self, width_px: u32, width_mm: f32) {
        self.pixel_density = width_px as f32 / width_mm;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

impl From<(u32, u32)> for PixelSize {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<PhysicalSize<u32>> for PixelSize {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<PixelSize> for (u32, u32) {
    fn from(val: PixelSize) -> Self {
        (val.width, val.height)
    }
}

/// Internal window state. This is used to store the winit window, the wgpu
/// device, the wgpu queue, etc.
#[derive(Dbg)]
pub struct WindowState {
    /// the winit window
    pub winit_window: Arc<winit::window::Window>,
    /// the wgpu surface
    pub surface: wgpu::Surface<'static>,
    /// the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
    /// the renderers
    #[dbg(placeholder = "[[ WgpuRenderer ]]")]
    pub wgpu_renderer: WgpuRenderer,
    #[dbg(placeholder = "[[ DynamicRenderer ]]")]
    pub renderer: DynamicRenderer,
    // The current mouse position. None if the mouse has left the window.
    pub mouse_position: Option<(f32, f32)>,
    /// Stores if the mouse cursor is currently visible.
    pub mouse_cursor_visible: bool,
    /// The size of the window in pixels.
    pub size: PixelSize,
    /// Physical properties of the screen.
    pub physical_screen: PhysicalScreen,
    /// Event handlers for the window.
    #[dbg(placeholder = "...")]
    pub event_handlers: HashMap<EventHandlerId, (EventKind, EventHandler)>,
}

unsafe impl Send for WindowState {}

impl WindowState {
    /// Resize the window's renders
    pub fn resize(&mut self, size: PixelSize, gpu_state: &mut GPUState) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;

        self.surface.configure(&gpu_state.device, &self.config);

        self.wgpu_renderer
            .resize(size.width, size.height, &self.surface, &gpu_state.device);
    }
}

/// How to block when presenting a frame.
/// A Window represents a window on the screen. It is used to create stimuli and
/// to submit them to the screen for rendering. Each window has a render task
/// that is responsible for rendering stimuli to the screen.
#[derive(Dbg, Clone)]
#[pyclass(unsendable)]
pub struct Window {
    /// Window ID
    pub winit_id: WindowId,
    /// The window state. Shared between all clones of the window.
    pub state: Arc<Mutex<WindowState>>,
    /// gpu state for the window
    pub gpu_state: Arc<Mutex<GPUState>>,
    /// Broadcast sender for keyboard events.
    pub event_broadcast_sender: async_broadcast::Sender<Event>,
    /// Broadcast receiver for keyboard events.
    pub event_broadcast_receiver: async_broadcast::InactiveReceiver<Event>,
}

impl Window {
    /// Creates a new physical input receiver that will receive physical input
    /// events from the window.
    pub fn create_event_receiver(&self) -> EventReceiver {
        EventReceiver {
            receiver: self.event_broadcast_receiver.activate_cloned(),
        }
    }

    /// Resizes the window's surface to the given size.
    pub fn resize(&self, size: impl Into<PixelSize>) {
        let size = size.into();
        let mut gpu_state = self.gpu_state.lock().unwrap();
        let mut win_state = self.state.lock().unwrap();

        win_state.resize(size, &mut gpu_state);
    }

    /// Present a frame on the window.
    pub fn present(&self, frame: &mut Frame) {
        // lock the gpu state and window state
        let gpu_state = &mut self.gpu_state.lock().unwrap();
        let mut win_state = &mut self.state.lock().unwrap();

        let device = &gpu_state.device;
        let queue = &gpu_state.queue;
        let width = win_state.size.width;
        let height = win_state.size.height;

        let config = &win_state.config;

        let suface_texture = win_state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let width = suface_texture.texture.size().width;
        let height = suface_texture.texture.size().height;

        let texture = win_state.wgpu_renderer.texture();

        win_state
            .renderer
            .render_to_texture(device, queue, texture, width, height, &mut frame.scene);

        let surface_texture_view = suface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(config.format),
            ..wgpu::TextureViewDescriptor::default()
        });

        // render the texture to the surface
        win_state
            .wgpu_renderer
            .render_to_texture(device, queue, &surface_texture_view);

        // present the frame
        suface_texture.present();
    }

    pub fn close(&self) {
        todo!()
    }

    /// Set the visibility of the mouse cursor.
    pub fn set_cursor_visible(&self, visible: bool) {
        let mut win_state = self.state.lock().unwrap();
        win_state.mouse_cursor_visible = visible;
        win_state.winit_window.set_cursor_visible(false);
    }

    /// Returns true if the mouse cursor is currently visible.
    pub fn cursor_visible(&self) -> bool {
        let win_state = &self.state.lock().unwrap();
        win_state.mouse_cursor_visible
    }

    /// Returns the mouse position. None if cursor not in window.
    pub fn mouse_position(&self) -> Option<(f32, f32)> {
        let win_state = &self.state.lock().unwrap();
        win_state.mouse_position.clone()
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
    pub fn size(&self) -> PixelSize {
        let win_state = &self.state.lock().unwrap();
        win_state.size
    }

    /// Return a new frame for the window.
    pub fn get_frame(&self) -> Frame {
        let win_state = &self.state.lock().unwrap();
        let scene = win_state
            .renderer
            .create_scene(win_state.size.width, win_state.size.height);
        Frame {
            bg_color: LinRgba::new(0.0, 0.0, 0.0, 1.0),
            scene,
            window: self.clone(),
        }
    }
    fn remove_event_handler(&self, id: EventHandlerId) {
        let mut state = self.state.lock().unwrap();
        state.event_handlers.remove(&id);
    }

    pub fn dispatch_event(&self, event: Event) -> bool {
        let mut handled = false;

        let event_handlers = {
            let state = self.state.lock().unwrap();

            // clone the event handlers
            let event_handlers = &state.event_handlers;

            let mut new_event_handlers: HashMap<EventHandlerId, (EventKind, EventHandler)> = HashMap::new();
            for (id, (kind, handler)) in event_handlers.iter() {
                new_event_handlers.insert(*id, (*kind, handler.clone()));
            }

            new_event_handlers
        };

        for (id, (kind, handler)) in event_handlers.iter() {
            // println!("Checking handler with id: {} for event kind: {:?}", id, kind);
            if kind == &event.kind() {
                // println!("Dispatching event to handler with id: {}", id);
                handled |= handler(event.clone());
                // println!("Handler with id: {} returned: {}", id, handled);
            }
        }

        handled
    }

    fn add_event_handler<F>(&self, kind: EventKind, handler: F) -> EventHandlerId
    where
        F: Fn(Event) -> bool + 'static + Send + Sync,
    {
        let mut state = self.state.lock().unwrap();
        let mut event_handlers = &mut state.event_handlers;

        // find a free id
        let id = loop {
            let id = rand::random::<EventHandlerId>();
            if !event_handlers.contains_key(&id) {
                break id;
            }
        };

        // add handler
        event_handlers.insert(id, (kind, Arc::new(handler)));

        id
    }

    pub fn lock_state(&self) -> MutexGuard<WindowState> {
        self.state.lock().unwrap()
    }
}

#[pymethods]
impl Window {
    #[pyo3(name = "get_frame")]
    fn py_get_frame(&self, py: Python) -> Frame {
        let self_wrapper = SendWrapper::new(self.clone());
        let d = py.allow_threads(move || SendWrapper::new(self_wrapper.get_frame()));
        d.take()
    }

    #[pyo3(name = "get_frames")]
    fn py_get_frames(&self, py: Python) -> FrameIterator {
        todo!()
    }

    #[pyo3(name = "present")]
    fn py_present(&self, frame: &mut Frame, py: Python) {
        let self_wrapper = SendWrapper::new(self.clone());
        let frame_wrapper = SendWrapper::new(frame);
        py.allow_threads(move || self_wrapper.present(frame_wrapper.take()));
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
        self.size().into()
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
    fn py_add_event_handler(&self, kind: EventKind, callback: Py<PyAny>, py: Python<'_>) {
        // let kind = EventKind::from_str(&kind).expect("Invalid event kind");

        let rust_callback_fn = move |event: Event| -> bool {
            Python::with_gil(|py| -> PyResult<()> {

                    callback.call1(py, (event,))
                            .expect("Error calling callback in event handler. Make sure the callback takes a single argument of type Event. Error");
                    Ok(())
                }).unwrap();
            false
        };

        let self_wrapper = SendWrapper::new(self);

        let id = self.add_event_handler(kind, rust_callback_fn);
    }

    /// Create a new EventReceiver that will receive events from the window.
    #[pyo3(name = "create_event_receiver")]
    fn py_create_event_receiver(&self) -> EventReceiver {
        self.create_event_receiver()
    }
}

/// FrameIterator is an iterator that yields frames.
#[derive(Debug, Clone)]
#[pyclass(unsendable)]
pub struct FrameIterator {
    /// The window that the frames are associated with.
    window: Window,
}

#[pymethods]
impl FrameIterator {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<FrameIterator>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<Frame>> {
        let frame = slf.window.get_frame();
        Ok(Some(frame))
    }
}

#[derive(Dbg)]
#[pyclass]
#[pyo3(unsendable)]
pub struct Frame {
    pub bg_color: super::color::LinRgba,
    #[dbg(placeholder = "...")]
    scene: DynamicScene,
    /// The window that the frame is associated with.
    window: Window,
}

impl Frame {
    /// Set the background color of the frame.
    pub fn set_bg_color(&mut self, bg_color: LinRgba) {
        self.bg_color = bg_color;
    }

    /// Draw onto the frame.
    pub fn draw(&mut self, stimulus: &DynamicStimulus) {
        let mut stimulus = stimulus.lock();

        let now = Instant::now();

        {
            // this needs to be scoped so that the mutable borrow of self is released
            let window_state = self.window.state.lock().unwrap();
            stimulus.update_animations(now, &window_state);
        }

        stimulus.draw(self);
    }

    pub fn window(&self) -> Window {
        self.window.clone()
    }

    pub fn scene(&self) -> &DynamicScene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut DynamicScene {
        &mut self.scene
    }
}

#[pymethods]
impl Frame {
    #[pyo3(name = "draw")]
    fn py_draw(&mut self, stimulus: crate::visual::stimuli::PyStimulus, py: Python) {
        let mut self_wrapper = SendWrapper::new(self);
        let stimulus_wrapper = SendWrapper::new(stimulus);
        py.allow_threads(move || self_wrapper.draw(stimulus_wrapper.as_super()));
    }

    #[getter(bg_color)]
    fn py_get_bg_color(&self) -> super::color::LinRgba {
        self.bg_color
    }

    #[setter(bg_color)]
    fn py_set_bg_color(&mut self, bg_color: super::color::LinRgba) {
        self.bg_color = bg_color;
    }
}
