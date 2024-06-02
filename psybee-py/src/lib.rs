use std::sync::Arc;

use psybee::audio::{AudioDevice, AudioStimulus};
use psybee::input::{Event, EventHandlingExt, EventKind, EventReceiver, EventVec, MouseButton};
use psybee::visual::geometry::{Circle, Rectangle, Size, ToVertices, Transformable, Transformation2D};
#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
use psybee::visual::stimuli::VideoStimulus;
use psybee::visual::stimuli::{GaborStimulus, ImageStimulus, SpriteStimulus, Stimulus};
use psybee::visual::window::Frame;
use psybee::visual::Window;
use psybee::{errors, ExperimentManager, MainLoop, Monitor, WindowOptions};
use pyo3::prelude::*;
use pyo3::{py_run, Python};
use pywrap::{py_forward, py_wrap, transmute_ignore_size};
use send_wrapper::SendWrapper;
use smol::lock::Mutex;

/// A list of `Stimulus` objects.
#[pyclass(frozen, name = "StimulusList")]
#[derive(Clone)]
pub struct PyStimulusList(Arc<Mutex<Vec<Box<dyn Stimulus + 'static>>>>);

#[pymethods]
impl PyStimulusList {
    fn __len__(&self) -> usize {
        self.0.lock_blocking().len()
    }

    fn __getitem__(&self, index: usize) -> PyStimulus {
        PyStimulus(dyn_clone::clone_box(&*self.0.lock_blocking()[index]))
    }

    fn __setitem__(&self, index: usize, value: PyStimulus) {
        self.0.lock_blocking()[index] = value.0;
    }

    fn append(&self, value: PyStimulus) {
        self.0.lock_blocking().push(value.0);
    }

    fn extend(&self, other: Vec<PyStimulus>) {
        self.0.lock_blocking().extend(other.into_iter().map(|s| s.0));
    }

    fn clear(&self) {
        self.0.lock_blocking().clear();
    }

    fn reverse(&self) {
        self.0.lock_blocking().reverse();
    }

    fn __repr__(&self) -> String {
        format!("<StimulusList with {} items>", self.0.lock_blocking().len())
    }

    fn remove(&self, value: PyStimulus) {
        let mut list = self.0.lock_blocking();
        let index = list.iter().position(|s| s.equal(&*value.0));
        if let Some(index) = index {
            list.remove(index);
        }
    }
}

py_wrap!(MainLoop, unsendable);

#[pymethods]
impl PyMainLoop {
    #[new]
    fn __new__() -> Self {
        PyMainLoop(smol::block_on(MainLoop::new()))
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn get_available_monitors(&mut self) -> Vec<PyMonitor> {
        self.0.get_available_monitors().iter().map(|m| PyMonitor(m.clone())).collect()
    }

    /// Runs your experiment function. This function will block the current thread
    /// until the experiment function returns.
    /// returns.
    ///
    /// Parameters
    /// ----------
    /// experiment_fn : callable
    ///    The function that runs your experiment. This function should take a single argument, an instance of `ExperimentManager`, and should not return nothing.
    fn run_experiment(&mut self, py: Python, experiment_fn: Py<PyAny>) {
        let rust_experiment_fn = move |wm: ExperimentManager| -> Result<(), errors::PsybeeError> {
            Python::with_gil(|py| -> PyResult<()> {
                let pywin = PyExperimentManager(wm);
                experiment_fn.call1(py, (pywin,)).expect("Error calling experiment_fn");
                Ok(())
            }).unwrap();
            Ok(())
        };

        let mut self_wrapper = SendWrapper::new(self);

        py.allow_threads(move || self_wrapper.0.run_experiment(rust_experiment_fn));
    }

    /// Prompt the user for input. This function will block the current thread
    /// until the user has entered a response.
    fn prompt(&self, prompt: &str, py: Python<'_>) -> String {
        let self_wrapper = SendWrapper::new(self);

        // prompt the user
        py.allow_threads(move || self_wrapper.0.prompt(prompt))
    }
}

py_wrap!(Monitor);

#[pymethods]
impl PyMonitor {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

py_wrap!(Window, module = "psybee_py.psybee_py");

#[pymethods]
impl PyWindow {
    /// Obtain the next frame for the window.
    ///
    /// Returns
    /// -------
    /// frame : Frame
    ///    The frame that was obtained from the window.
    fn get_frame(&self) -> PyFrame {
        let self_wrapper = SendWrapper::new(self);
        PyFrame(self_wrapper.0.get_frame())
    }

    /// Submit a frame to the window. Might or might not block, depending
    /// on the current state of the underlying GPU queue.
    ///
    /// Parameters
    /// ----------
    /// frame : Frame
    ///   The frame to submit to the window.
    fn present(&self, frame: &PyFrame, py: Python<'_>) {
        let self_wrapper = SendWrapper::new(self);
        py.allow_threads(move || {
              self_wrapper.0.present(frame.0.clone(), None);
          });
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn close(&self) {
        self.0.close();
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
    fn add_event_handler(&self, kind: PyEventKind, callback: Py<PyAny>, py: Python<'_>) {
        let rust_callback_fn = move |event: Event| -> bool {
            Python::with_gil(|py| -> PyResult<()> {
                let pyevent = PyEvent(event);
                callback.call1(py, (pyevent,))
                        .expect("Error calling callback in event handler. Make sure the callback takes a single argument of type Event. Error");
                Ok(())
            }).unwrap();
            false
        };

        let self_wrapper = SendWrapper::new(self);

        // give up the GIL and
        // run the experiment
        py.allow_threads(move || self_wrapper.0.add_event_handler(kind.into(), rust_callback_fn));
    }

    /// Stimuli that are currently attached to the window.
    #[getter]
    fn stimuli(&self) -> PyStimulusList {
        PyStimulusList(self.0.stimuli.clone())
    }

    /// Create a new EventReceiver for the window. The EventReceiver can be used
    /// to poll for events that have occurred on the window.
    ///
    /// Returns
    /// -------
    /// receiver : EventReceiver
    ///     The EventReceiver that was created.
    fn create_event_receiver(&self) -> PyEventReceiver {
        PyEventReceiver(self.0.create_event_receiver())
    }

    /// Set the visibility of the cursor.
    #[setter]
    fn set_cursor_visible(&self, visible: bool) {
        let self_wrapper = SendWrapper::new(self);

        // set the cursor
        Python::with_gil(|py| {
            py.allow_threads(move || self_wrapper.0.set_cursor_visible(visible));
        });
    }

    /// Visible state of the cursor.
    #[getter]
    fn cursor_visible(&self) -> bool {
        self.0.cursor_visible()
    }

    /// The width of the window in pixels.
    #[getter]
    fn width_px(&self) -> u32 {
        self.0.width_px()
    }

    /// The height of the window in pixels.
    #[getter]
    fn height_px(&self) -> u32 {
        self.0.height_px()
    }
}
py_wrap!(EventReceiver);

#[pymethods]
impl PyEventReceiver {
    fn poll(&mut self) -> PyEventVec {
        PyEventVec { vec: self.0.poll(),
                     i: Arc::new(Mutex::new(0)) }
    }
}

#[pyclass(name = "EventVec")]
#[derive(Clone)]
pub struct PyEventVec {
    vec: EventVec,
    i: Arc<Mutex<usize>>,
}

#[pymethods]
impl PyEventVec {
    /// Convinience method to check if a key was pressed in the event vector.
    fn key_pressed(&self, key: &str) -> bool {
        self.vec.iter().any(|key_event| key_event.key_pressed(key))
    }

    /// Convinience method to check if a key was released in the event vector.
    fn key_released(&self, key: &str) -> bool {
        self.vec.iter().any(|key_event| key_event.key_released(key))
    }

    fn __len__(&self) -> usize {
        self.vec.len()
    }

    fn __getitem__(&self, index: usize) -> PyEvent {
        PyEvent(self.vec[index].clone())
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        return slf;
    }

    fn __next__(slf: PyRef<'_, Self>) -> Option<PyEvent> {
        let mut i = slf.i.lock_blocking();
        if *i < slf.vec.len() {
            let event = PyEvent(slf.vec[*i].clone());
            *i += 1;
            Some(event)
        } else {
            None
        }
    }
}

#[pyo3::prelude::pyclass(name = "Frame")]
/// A Frame that can be used to render stimuli.
pub struct PyFrame(pub Frame);

pub trait DummyTrait1Frame {}
impl DummyTrait1Frame for Frame {}

impl<T> From<&T> for PyFrame where T: DummyTrait1Frame + Clone
{
    fn from(inner: &T) -> Self {
        let inner = inner.clone();
        let inner = transmute_ignore_size!(inner);
        Self(inner)
    }
}

#[pymethods]
impl PyFrame {
    /// Set the background color of the frame.
    fn set_bg_color(&mut self, color: (f32, f32, f32)) {
        self.0.set_bg_color(psybee::visual::color::SRGBA::new(color.0, color.1, color.2, 1.0));
    }

    #[getter]
    /// The stimuli that are currently attached to the frame.
    fn stimuli(&self) -> PyStimulusList {
        PyStimulusList(self.0.stimuli.clone())
    }
}

py_wrap!(WindowOptions);

#[pymethods]
impl PyWindowOptions {
    /// Create a new
    /// WindowOptions object.
    #[new]
    #[pyo3(signature = (mode = "windowed", resolution = None, monitor = None, refresh_rate = None))]
    fn __new__(mode: &str, resolution: Option<(u32, u32)>, monitor: Option<&PyMonitor>, refresh_rate: Option<f64>) -> Self {
        match mode {
            "windowed" => PyWindowOptions(WindowOptions::Windowed { resolution: None }),
            "fullscreen_exact" => PyWindowOptions(WindowOptions::FullscreenExact { resolution: resolution,
                                                                                   monitor: monitor.map(|m| m.0.clone()),
                                                                                   refresh_rate: refresh_rate }),
            "fullscreen_highest_resolution" => PyWindowOptions(WindowOptions::FullscreenHighestResolution { monitor: monitor.map(|m| m.0.clone()),
                                                                                                            refresh_rate: refresh_rate }),
            "fullscreen_highest_refresh_rate" => PyWindowOptions(WindowOptions::FullscreenHighestRefreshRate { monitor: monitor.map(|m| m.0.clone()),
                                                                                                               resolution: resolution }),
            _ => panic!("Unknown mode: {}", mode),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

py_wrap!(ExperimentManager);
py_forward!(ExperimentManager, fn prompt(&self, prompt: &str) -> String);

#[pymethods]
impl PyExperimentManager {
    fn create_default_window(&self, py: Python<'_>) -> PyWindow {
        let self_wrapper = SendWrapper::new(self);
        py.allow_threads(move || PyWindow(self_wrapper.0.create_default_window()))
    }
}

// STIMULI
// A generic shape that wrap
// anything that implements
// the ToVertices trait
#[pyclass(subclass, name = "Shape", module = "psybee_py.psybee_py")]
pub struct PyShape(Box<dyn psybee::visual::geometry::ToVertices>);

impl PyShape {
    fn new(shape: Box<dyn psybee::visual::geometry::ToVertices>) -> Self {
        PyShape(shape)
    }
}

impl ToVertices for PyShape {
    fn to_vertices_px(&self, screenwidth_mm: f64, viewing_distance_mm: f64, width_px: u32, height_px: u32) -> Vec<psybee::visual::geometry::Vertex> {
        self.0.to_vertices_px(screenwidth_mm, viewing_distance_mm, width_px, height_px)
    }

    fn clone_box(&self) -> Box<dyn ToVertices> {
        self.0.clone_box()
    }

    fn contains(&self, window: &Window, transform: &Transformation2D, x: Size, y: Size) -> bool {
        self.0.contains(window, transform, x, y)
    }
}

// A Rectangle (a type of
// shape)
#[pyclass(name = "Rectangle", extends = PyShape, module = "psybee_py.psybee_py")]
pub struct PyRectangle();

#[pymethods]
impl PyRectangle {
    #[new]
    fn __new__(x: PySize, y: PySize, width: PySize, height: PySize) -> (Self, PyShape) {
        (PyRectangle(), PyShape::new(Box::new(Rectangle::new(x.0, y.0, width.0, height.0))))
    }

    /// Create a fullscreen rectangle
    ///
    /// Returns
    /// -------
    /// Rectangle :
    ///   The fullscreen rectangle that was created.
    #[staticmethod]
    fn fullscreen(py: Python<'_>) -> PyObject {
        // (PyRectangle(),
        // PyShape::new(Box::new(Rectangle::FULLSCREEN)))
        let o = PyClassInitializer::from(PyShape::new(Box::new(Rectangle::FULLSCREEN))).add_subclass(PyRectangle());

        Py::new(py, o).unwrap().to_object(py)
    }
}

/// A Circle shape defined by a center (x, y) and a radius.
#[pyclass(name = "Circle", extends = PyShape)]
pub struct PyCircle();

#[pymethods]
impl PyCircle {
    #[new]
    fn __new__(x: PySize, y: PySize, radius: PySize) -> (Self, PyShape) {
        (PyCircle(), PyShape::new(Box::new(Circle::new(x.0, y.0, radius.0))))
    }
}

/// Represents a Stimulus. This class is used either as a base class for other
/// stimulus classes or as a standalone class, when no specific runtume type
/// information is available.
#[pyclass(name = "Stimulus", subclass, frozen, module = "psybee_py.psybee_py")]
pub struct PyStimulus(Box<dyn Stimulus + 'static>);

#[pymethods]
impl PyStimulus {
    fn contains(&self, x: PySize, y: PySize, py: Python<'_>) -> bool {
        let self_wrapper = SendWrapper::new(self);
        py.allow_threads(move || self_wrapper.0.contains(x.0, y.0))
    }

    #[setter]
    fn set_visible(&self, is_visible: bool) {
        self.0.set_visible(is_visible);
    }

    #[getter]
    fn visible(&self) -> bool {
        self.0.visible()
    }

    fn toggle_visibility(&self) {
        self.0.toggle_visibility();
    }

    fn hide(&self) {
        self.0.hide();
    }

    fn show(&self) {
        self.0.show();
    }
}

impl Clone for PyStimulus {
    fn clone(&self) -> Self {
        PyStimulus(dyn_clone::clone_box(&*self.0))
    }
}

/// A VideoStimulus.
#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
#[pyclass(name = "VideoStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyVideoStimulus();

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
#[pymethods]
impl PyVideoStimulus {
    #[new]
    fn __new__(window: &PyWindow, shape: &PyShape, path: &str, width: u32, height: u32, thumbnail: Option<f32>, init: Option<bool>) -> (Self, PyStimulus) {
        let stim = psybee::visual::stimuli::VideoStimulus::new(&window.0, shape.0.clone_box(), path, width, height, thumbnail, init);
        (PyVideoStimulus(), PyStimulus(Box::new(stim)))
    }

    // insteaf of self, we
    // take a mutable
    // reference to the
    // PyVideoStimulus
    fn init(slf: PyRef<'_, Self>) {
        // downcast the
        // TraibObject:
        // Stmulus to a
        // VideoStimulus
        slf.into_super().0.downcast_ref::<VideoStimulus>().expect("Failed to downcast to VideoStimulus").init();
    }

    fn play(slf: PyRef<'_, Self>) {
        slf.into_super().0.downcast_ref::<VideoStimulus>().expect("Failed to downcast to VideoStimulus").play();
    }

    fn pause(slf: PyRef<'_, Self>) {
        slf.into_super().0.downcast_ref::<VideoStimulus>().expect("Failed to downcast to VideoStimulus").pause();
    }
}

/// An ImageStimulus.
#[pyclass(name = "ImageStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyImageStimulus();

#[pymethods]
impl PyImageStimulus {
    #[new]
    fn __new__(window: &PyWindow, shape: &PyShape, path: &str, py: Python<'_>) -> (Self, PyStimulus) {
        let _self_wrapper = SendWrapper::new(window);
        py.allow_threads(move || {
              let stim = ImageStimulus::new(&window.0, shape.0.clone_box(), path);
              (PyImageStimulus(), PyStimulus(Box::new(stim)))
          })
    }

    fn set_translation(slf: PyRef<'_, Self>, x: PySize, y: PySize) {
        slf.into_super()
           .0
           .downcast_ref::<ImageStimulus>()
           .expect("Failed to downcast to ImageStimulus")
           .set_translation(x.0, y.0);
    }
}

/// A SpriteStimulus that can be used to display an animated sprite.
#[pyclass(name = "SpriteStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PySpriteStimulus();

#[pymethods]
impl PySpriteStimulus {
    #[new]
    fn __new__(window: &PyWindow, shape: &PyShape, sprite_path: &str, num_sprites_x: u32, num_sprites_y: u32, fps: Option<f64>, repeat: Option<u64>) -> (Self, PyStimulus) {
        let stim = SpriteStimulus::new_from_spritesheet(&window.0, shape.0.clone_box(), sprite_path, num_sprites_x, num_sprites_y, fps, repeat);
        (PySpriteStimulus(), PyStimulus(Box::new(stim)))
    }

    fn advance_image_index(slf: PyRef<'_, Self>) {
        slf.into_super()
           .0
           .downcast_ref::<SpriteStimulus>()
           .expect("Failed to downcast to SpriteStimulus")
           .advance_image_index();
    }

    fn reset(slf: PyRef<'_, Self>) {
        slf.into_super().0.downcast_ref::<SpriteStimulus>().expect("Failed to downcast to SpriteStimulus").reset();
    }

    fn set_translation(slf: PyRef<'_, Self>, x: PySize, y: PySize) {
        slf.into_super()
           .0
           .downcast_ref::<SpriteStimulus>()
           .expect("Failed to downcast to SpriteStimulus")
           .set_translation(x.0, y.0);
    }
}

/// A GaborStimulus.
///
/// Consists of a Gabor patch, which is a sinusoidal grating enveloped by a
/// Gaussian envelope.
///
/// Parameters
/// ----------
/// window : Window
///     The window that the stimulus will be presented on.
/// shape : Shape
///     The shape of the stimulus.
/// phase : float
///     The phase of the sinusoidal grating in radians.
/// cycle_length : Size   
///     The length of a single cycle of the sinusoidal grating.
/// std_x : Size
///     The standard deviation of the Gaussian envelope in the x direction.
/// std_y : Size
///     The standard deviation of the Gaussian envelope in the y direction in pixels.
/// orientation : float
///     The orientation of the sinusoidal grating in adians.
/// color : tuple
///  The color of the stimulus as an RGB tuple.
///
/// Returns
/// -------
/// GaborStimulus :
///  The GaborStimulus that was created.
#[pyclass(name = "GaborStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyGaborStimulus();

#[pymethods]
impl PyGaborStimulus {
    #[new]
    fn __new__(window: &PyWindow,
               shape: &PyShape,
               phase: f32,
               cycle_length: PySize,
               std_x: PySize,
               std_y: PySize,
               orientation: f32,
               color: (f32, f32, f32),
               py: Python<'_>)
               -> (Self, PyStimulus) {
        let _self_wrapper = SendWrapper::new(window);
        py.allow_threads(move || {
              let stim = GaborStimulus::new(&window.0,
                                            shape.0.clone_box(),
                                            phase,
                                            cycle_length.0.clone(),
                                            std_x.0.clone(),
                                            std_y.0.clone(),
                                            orientation,
                                            psybee::visual::color::SRGBA::new(color.0, color.1, color.2, 1.0));
              (PyGaborStimulus(), PyStimulus(Box::new(stim)))
          })
    }

    fn set_phase(slf: PyRef<'_, Self>, phase: f32) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .set_phase(phase);
    }

    fn phase(slf: PyRef<'_, Self>) -> f32 {
        slf.into_super().0.downcast_ref::<GaborStimulus>().expect("Failed to downcast to GaborStimulus").phase()
    }

    fn set_cycle_length(slf: PyRef<'_, Self>, cycle_length: PySize) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .set_cycle_length(cycle_length.0);
    }

    fn cycle_length(slf: PyRef<'_, Self>) -> PySize {
        PySize(slf.into_super()
                  .0
                  .downcast_ref::<GaborStimulus>()
                  .expect("Failed to downcast to GaborStimulus")
                  .cycle_length())
    }

    fn set_color(slf: PyRef<'_, Self>, color: (f32, f32, f32)) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .set_color(psybee::visual::color::SRGBA::new(color.0, color.1, color.2, 1.0));
    }

    fn color(slf: PyRef<'_, Self>) -> (f32, f32, f32) {
        let color = slf.into_super().0.downcast_ref::<GaborStimulus>().expect("Failed to downcast to GaborStimulus").color();
        (color.r, color.g, color.b)
    }

    fn set_orientation(slf: PyRef<'_, Self>, orientation: f32) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .set_orientation(orientation);
    }

    fn orientation(slf: PyRef<'_, Self>) -> f32 {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .orientation()
    }

    fn translate(slf: PyRef<'_, Self>, x: PySize, y: PySize) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .translate(x.0, y.0);
    }

    fn set_translation(slf: PyRef<'_, Self>, x: PySize, y: PySize) {
        slf.into_super()
           .0
           .downcast_ref::<GaborStimulus>()
           .expect("Failed to downcast to GaborStimulus")
           .set_translation(x.0, y.0);
    }
}

py_wrap!(AudioDevice, unsendable);
py_forward!(AudioDevice, fn new() -> AudioDevice);

/// An AudioStimulus.
#[pyclass(name = "AudioStimulus", subclass)]
pub struct PyAudioStimulus(Box<dyn AudioStimulus + 'static>);

#[pymethods]
impl PyAudioStimulus {
    fn reset(&mut self) {
        self.0.reset();
    }

    fn set_volume(&mut self, volume: f32) {
        self.0.set_volume(volume);
    }

    fn play(&mut self) {
        self.0.play();
    }

    fn pause(&mut self) {
        self.0.pause();
    }

    fn seek(&mut self, time: f32) {
        self.0.seek(time);
    }

    fn restart(&mut self) {
        self.0.restart();
    }
}

/// A SineWaveStimulus.
///
/// Parameters
/// ----------
/// audio_device : AudioDevice
///    The audio device that the stimulus will be played on.
/// frequency : float
///   The frequency of the sine wave in Hz.
/// duration : float
///  The duration of the stimulus in seconds.
#[pyclass(name = "SineWaveStimulus", extends = PyAudioStimulus)]
#[derive(Clone)]
pub struct PySineWaveStimulus();

#[pymethods]
impl PySineWaveStimulus {
    #[new]
    fn __new__(audio_device: &PyAudioDevice, frequency: f32, duration: f32) -> (Self, PyAudioStimulus) {
        let stim = psybee::audio::SineWaveStimulus::new(&audio_device.0, frequency, duration);
        (PySineWaveStimulus(), PyAudioStimulus(Box::new(stim)))
    }
}

/// An audio stimulus that plays a sound from a file. See the `rodio` crate for
/// supported file formats.
///
/// Parameters
/// ----------
/// audio_device : AudioDevice
///   The audio device that the stimulus will be played on.
/// file_path : str
///  The path to the audio file that will be played.
#[pyclass(name = "FileStimulus", extends = PyAudioStimulus)]
#[derive(Clone)]
pub struct PyFileStimulus();

#[pymethods]
impl PyFileStimulus {
    #[new]
    fn __new__(audio_device: &PyAudioDevice, file_path: &str) -> (Self, PyAudioStimulus) {
        let stim = psybee::audio::FileStimulus::new(&audio_device.0, file_path);
        (PyFileStimulus(), PyAudioStimulus(Box::new(stim)))
    }
}

// /// The TextStimulus
// ///
// /// Parameters
// /// ----------
// /// window : PyWindow
// ///     The window that the stimulus will be presented on.
// /// text : str
// ///     The text to be displayed.
// ///
// /// Returns
// /// -------
// /// TextStimulus :
// ///     The TextStimulus that was created.
// ///
// /// Examples
// /// --------
// /// >>> window = PyWindow()
// /// >>> text_stimulus = PyTextStimulus(window, "Hello, World!")

/// A generic size.
#[pyo3::prelude::pyclass(name = "Size", subclass)]
pub struct PySize(pub Size);

impl Clone for PySize {
    fn clone(&self) -> Self {
        PySize(self.0.clone())
    }
}

#[pymethods]
impl PySize {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __add__(&self, other: PySize) -> PySize {
        PySize(self.0.clone() + other.0.clone())
    }

    fn __sub__(&self, other: PySize) -> PySize {
        PySize(self.0.clone() - other.0.clone())
    }

    fn __mul__(&self, other: f64) -> PySize {
        PySize(self.0.clone() * other)
    }

    fn __div__(&self, other: f64) -> PySize {
        PySize(self.0.clone() / other)
    }
}

/// A `Size` in pixels.
///
/// Parameters
/// ----------
/// value : float
///    The value of the size in pixels.
#[pyclass(name = "Pixels", extends = PySize)]
pub struct PyPixels();

#[pymethods]
impl PyPixels {
    #[new]
    fn __new__(value: f64) -> (Self, PySize) {
        (PyPixels(), PySize(Size::Pixels(value)))
    }
}

// ScreenWidth
#[pyclass(name = "ScreenWidth", extends = PySize)]
pub struct PyScreenWidth();

#[pymethods]
impl PyScreenWidth {
    #[new]
    fn __new__(value: f64) -> (Self, PySize) {
        (PyScreenWidth(), PySize(Size::ScreenWidth(value)))
    }
}

// ScreenHeight
#[pyclass(name = "ScreenHeight", extends = PySize)]
pub struct PyScreenHeight();

#[pymethods]
impl PyScreenHeight {
    #[new]
    fn __new__(value: f64) -> (Self, PySize) {
        (PyScreenHeight(), PySize(Size::ScreenHeight(value)))
    }
}

#[pyclass(name = "Event")]
pub struct PyEvent(Event);

#[pyclass(name = "EventKind", rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Clone)]
pub enum PyEventKind {
    KeyPress,
    KeyRelease,
    MouseButtonPress,
    MouseButtonRelease,
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
    FocusGained,
    FocusLost,
    CursorMoved,
    CursorEntered,
    CursorExited,
    TouchpadPress,
    MouseWheel,
    Other,
}

impl From<EventKind> for PyEventKind {
    fn from(kind: EventKind) -> Self {
        match kind {
            EventKind::KeyPress => PyEventKind::KeyPress,
            EventKind::KeyRelease => PyEventKind::KeyRelease,
            EventKind::MouseButtonPress => PyEventKind::MouseButtonPress,
            EventKind::MouseButtonRelease => PyEventKind::MouseButtonRelease,
            EventKind::TouchStart => PyEventKind::TouchStart,
            EventKind::TouchMove => PyEventKind::TouchMove,
            EventKind::TouchEnd => PyEventKind::TouchEnd,
            EventKind::TouchCancel => PyEventKind::TouchCancel,
            EventKind::FocusGained => PyEventKind::FocusGained,
            EventKind::FocusLost => PyEventKind::FocusLost,
            EventKind::CursorMoved => PyEventKind::CursorMoved,
            EventKind::CursorEntered => PyEventKind::CursorEntered,
            EventKind::CursorExited => PyEventKind::CursorExited,
            EventKind::TouchpadPress => PyEventKind::TouchpadPress,
            EventKind::MouseWheel => PyEventKind::MouseWheel,
            EventKind::Other => PyEventKind::Other,
        }
    }
}

impl From<PyEventKind> for EventKind {
    fn from(kind: PyEventKind) -> Self {
        match kind {
            PyEventKind::KeyPress => EventKind::KeyPress,
            PyEventKind::KeyRelease => EventKind::KeyRelease,
            PyEventKind::MouseButtonPress => EventKind::MouseButtonPress,
            PyEventKind::MouseButtonRelease => EventKind::MouseButtonRelease,
            PyEventKind::TouchStart => EventKind::TouchStart,
            PyEventKind::TouchMove => EventKind::TouchMove,
            PyEventKind::TouchEnd => EventKind::TouchEnd,
            PyEventKind::TouchCancel => EventKind::TouchCancel,
            PyEventKind::FocusGained => EventKind::FocusGained,
            PyEventKind::FocusLost => EventKind::FocusLost,
            PyEventKind::CursorMoved => EventKind::CursorMoved,
            PyEventKind::CursorEntered => EventKind::CursorEntered,
            PyEventKind::CursorExited => EventKind::CursorExited,
            PyEventKind::TouchpadPress => EventKind::TouchpadPress,
            PyEventKind::MouseWheel => EventKind::MouseWheel,
            PyEventKind::Other => EventKind::Other,
        }
    }
}

#[pymethods]
impl PyEvent {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    #[getter]
    fn timestamp(&self) -> f64 {
        // convert to f64
        // (seconds since
        // epoch)
        self.0.timestamp().duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_secs_f64()
    }

    #[getter]
    fn kind(&self) -> PyEventKind {
        return self.0.kind().into();
    }

    #[getter]
    fn position(&self) -> Option<(PySize, PySize)> {
        self.0.position().map(|p| (PySize(p.0.clone()), PySize(p.1.clone())))
    }
}

// #[pyo3::prelude::pyclass(name = "MouseButton")]
// #[derive(Debug, Clone)]
// pub struct
// PyMouseButton(pub
// MouseButton);

#[pyclass(name = "MouseButton", get_all)]
#[derive(Debug, Clone)]
pub enum PyMouseButton {
    Left {},
    Right {},
    Middle {},
    Forward {},
    Back {},
    Other { index: u16 },
}

impl From<MouseButton> for PyMouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => PyMouseButton::Left {},
            MouseButton::Right => PyMouseButton::Right {},
            MouseButton::Middle => PyMouseButton::Middle {},
            MouseButton::Forward => PyMouseButton::Forward {},
            MouseButton::Back => PyMouseButton::Back {},
            MouseButton::Other(index) => PyMouseButton::Other { index },
        }
    }
}

#[pyclass(name = "EventData", get_all)]
pub enum PyEventData {
    KeyPress { key: String, code: u32 },
    KeyRelease { key: String, code: u32 },
    MouseButtonPress { button: PyMouseButton, position: (PySize, PySize) },
    CursorMoved { position: (PySize, PySize) },
    Other { desc: String },
}

// Handling for user input

#[pymodule]
#[pyo3(name = "psybee")]
fn psybee_py(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    // init simplelog to file
    //simplelog::WriteLogger::init(simplelog::LevelFilter::Warn, simplelog::Config::default(), std::fs::File::create("C:/Users/psyphyuser/Documents/psybee.log").unwrap()).unwrap();

    pyo3::prepare_freethreaded_python();

    m.add_class::<PyExperimentManager>()?;
    m.add_class::<PyMonitor>()?;
    m.add_class::<PyMainLoop>()?;

    let mod_window = PyModule::new_bound(py, "window")?;
    py_run!(py, mod_window, "import sys; sys.modules['psybee.window'] = mod_window");
    mod_window.add_class::<PyWindowOptions>()?;
    mod_window.add_class::<PyWindow>()?;
    mod_window.add_class::<PyFrame>()?;

    let mod_geometry = PyModule::new_bound(py, "geometry")?;
    py_run!(py, mod_geometry, "import sys; sys.modules['psybee.geometry'] = mod_geometry");
    mod_geometry.add_class::<PyShape>()?;
    mod_geometry.add_class::<PyRectangle>()?;
    mod_geometry.add_class::<PyCircle>()?;
    mod_geometry.add_class::<PySize>()?;
    mod_geometry.add_class::<PyPixels>()?;
    mod_geometry.add_class::<PyScreenWidth>()?;
    mod_geometry.add_class::<PyScreenHeight>()?;

    let mod_stimuli = PyModule::new_bound(py, "stimuli")?;
    py_run!(py, mod_stimuli, "import sys; sys.modules['psybee.stimuli'] = mod_stimuli");
    mod_stimuli.add_class::<PyGaborStimulus>()?;
    mod_stimuli.add_class::<PyImageStimulus>()?;
    mod_stimuli.add_class::<PySpriteStimulus>()?;
    mod_stimuli.add_class::<PyStimulus>()?;
    #[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
    mod_stimuli.add_class::<PyVideoStimulus>()?;

    let mod_events = PyModule::new_bound(py, "events")?;
    py_run!(py, mod_events, "import sys; sys.modules['psybee.events'] = mod_events");
    mod_events.add_class::<PyEventReceiver>()?;
    mod_events.add_class::<PyEventVec>()?;
    mod_events.add_class::<PyEvent>()?;
    mod_events.add_class::<PyEventKind>()?;

    let mod_audio = PyModule::new_bound(py, "audio")?;
    py_run!(py, mod_audio, "import sys; sys.modules['psybee.audio'] = mod_audio");
    mod_audio.add_class::<PyAudioDevice>()?;
    mod_audio.add_class::<PyAudioStimulus>()?;
    mod_audio.add_class::<PySineWaveStimulus>()?;
    mod_audio.add_class::<PyFileStimulus>()?;

    m.add_submodule(&mod_window)?;
    m.add_submodule(&mod_geometry)?;
    m.add_submodule(&mod_stimuli)?;
    m.add_submodule(&mod_events)?;
    m.add_submodule(&mod_audio)?;

    Ok(())
}
