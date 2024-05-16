use psychophysics::input::Event;
use psychophysics::input::EventData;
use psychophysics::input::EventReceiver;
use psychophysics::input::MouseButton;

use psychophysics::{
    errors,
    input::EventVec,
    visual::{
        geometry::{Circle, Rectangle, Size, ToVertices, Transformable},
        stimuli::{
            GaborStimulus, ImageStimulus, SpriteStimulus,
            Stimulus,
        },
        window::{Frame, WindowState},
        Window,
    },
    wgpu, ExperimentManager, GPUState, Monitor, WindowManager, WindowOptions,
};
use pywrap::py_forward;
use pywrap::py_wrap;
use pywrap::transmute_ignore_size;


#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
use psychophysics::visual::stimuli::VideoStimulus;

use pyo3::{prelude::*, Python};
use send_wrapper::SendWrapper;

py_wrap!(ExperimentManager, unsendable);

#[pymethods]
impl PyExperimentManager {
    #[new]
    fn __new__() -> Self {
        PyExperimentManager(smol::block_on(ExperimentManager::new()))
    }

    fn __repr__(&self) -> String {
        // return the Debug representation of the ExperimentManager
        format!("{:?}", self.0)
    }

    fn get_available_monitors(&mut self) -> Vec<PyMonitor> {
        self.0
            .get_available_monitors()
            .iter()
            .map(|m| PyMonitor(m.clone()))
            .collect()
    }

    fn run_experiment(&mut self, py: Python, experiment_fn: Py<PyAny>) {
        let rust_experiment_fn =
            move |wm: WindowManager| -> Result<(), errors::PsychophysicsError> {
                Python::with_gil(|py| -> PyResult<()> {
                    let pywin = PyWindowManager(wm);
                    experiment_fn
                        .call1(py, (pywin,))
                        .expect("Error calling experiment_fn");
                    Ok(())
                })
                .unwrap();
                Ok(())
            };

        // wrap self in a SendWrapper so that it can be sent through the magic barrier
        let mut self_wrapper = SendWrapper::new(self);

        // give up the GIL and run the experiment
        py.allow_threads(move || self_wrapper.0.run_experiment(rust_experiment_fn));
    }

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
        // return the Debug representation of the Monitor
        format!("{:?}", self.0)
    }
}

py_wrap!(Window);

#[pymethods]
impl PyWindow {
    fn get_frame(&self) -> PyFrame {
        PyFrame(self.0.get_frame())
    }

    fn submit_frame(&mut self, frame: &PyFrame, py: Python<'_>) {
        let self_wrapper = SendWrapper::new(self);
        // submit the frame
        py.allow_threads(move || {
            self_wrapper.0.submit_frame(frame.0.clone());
        });
    }

    fn __repr__(&self) -> String {
        // return the Debug representation of the Window
        format!("{:?}", self.0)
    }

    fn close(&mut self) {
        self.0.close();
    }

    fn create_event_receiver(&self) -> PyEventReceiver {
        PyEventReceiver(self.0.create_event_receiver())
    }
}
py_wrap!(EventReceiver);
py_wrap!(EventVec);

#[pymethods]
impl PyEventReceiver {
    fn events(&mut self) -> PyEventVec {
        PyEventVec(self.0.events())
    }
}

#[pymethods]
impl PyEventVec {
    fn key_pressed(&self, key: &str) -> bool {
        self.0.iter().any(|key_event| key_event.key_pressed(key))
    }

    fn key_released(&self, key: &str) -> bool {
        self.0.iter().any(|key_event| key_event.key_released(key))
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    // allow indexing into the EventVec
    fn __getitem__(&self, index: usize) -> PyEvent {
        PyEvent(self.0[index].clone())
    }
}

py_wrap!(Frame);

#[pymethods]
impl PyFrame {
    fn set_bg_color(&mut self, color: (f32, f32, f32)) {
        self.0
            .set_bg_color(psychophysics::visual::color::SRGBA::new(
                color.0, color.1, color.2, 1.0,
            ));
    }

    fn add(&mut self, stim: &PyAny) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
        if let Ok(stim) = stim.extract::<PyVideoStimulus>() {
            self.0.add(&stim);
            return;
        }

        if let Ok(stim) = stim.extract::<PyGaborStimulus>() {
            self.0.add(&stim);
            return;
        }

        if let Ok(stim) = stim.extract::<PyImageStimulus>() {
            self.0.add(&stim);
            return;
        }

        if let Ok(stim) = stim.extract::<PySpriteStimulus>() {
            self.0.add(&stim);
            return;
        }

        panic!("Unknown stimulus type");
    }
}

py_wrap!(WindowOptions);

#[pymethods]
impl PyWindowOptions {
    /// Create a new WindowOptions object.
    #[new]
    #[pyo3(signature = (mode = "windowed", resolution = None, monitor = None, refresh_rate = None))]
    fn __new__(
        mode: &str,
        resolution: Option<(u32, u32)>,
        monitor: Option<&PyMonitor>,
        refresh_rate: Option<f64>,
    ) -> Self {
        match mode {
            "windowed" => PyWindowOptions(WindowOptions::Windowed { resolution: None }),
            "fullscreen_exact" => PyWindowOptions(WindowOptions::FullscreenExact {
                resolution: resolution,
                monitor: monitor.map(|m| m.0.clone()),
                refresh_rate: refresh_rate,
            }),
            "fullscreen_highest_resolution" => {
                PyWindowOptions(WindowOptions::FullscreenHighestResolution {
                    monitor: monitor.map(|m| m.0.clone()),
                    refresh_rate: refresh_rate,
                })
            }
            "fullscreen_highest_refresh_rate" => {
                PyWindowOptions(WindowOptions::FullscreenHighestRefreshRate {
                    monitor: monitor.map(|m| m.0.clone()),
                    resolution: resolution,
                })
            }
            _ => panic!("Unknown mode: {}", mode),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

py_wrap!(WindowManager);
py_forward!(WindowManager, fn prompt(&self, prompt: &str) -> String);

#[pymethods]
impl PyWindowManager {
    fn create_default_window(&self, py: Python<'_>) -> PyWindow {
        let self_wrapper = SendWrapper::new(self);
        py.allow_threads(move || PyWindow(self_wrapper.0.create_default_window()))
    }
}

// STIMULI
// A generic shape that wrap anything that implements the ToVertices trait
#[pyclass(subclass, name = "Shape")]
pub struct PyShape(Box<dyn psychophysics::visual::geometry::ToVertices>);

impl PyShape {
    fn new(shape: Box<dyn psychophysics::visual::geometry::ToVertices>) -> Self {
        PyShape(shape)
    }
}

impl ToVertices for PyShape {
    fn to_vertices_px(
        &self,
        screenwidth_mm: f64,
        viewing_distance_mm: f64,
        width_px: u32,
        height_px: u32,
    ) -> Vec<psychophysics::visual::geometry::Vertex> {
        self.0
            .to_vertices_px(screenwidth_mm, viewing_distance_mm, width_px, height_px)
    }

    fn clone_box(&self) -> Box<dyn ToVertices> {
        self.0.clone_box()
    }
}

// A Rectangle (a type of shape)
#[pyclass(name = "Rectangle", extends = PyShape)]
pub struct PyRectangle();

#[pymethods]
impl PyRectangle {
    #[new]
    fn __new__(x: PySize, y: PySize, width: PySize, height: PySize) -> (Self, PyShape) {
        (
            PyRectangle(),
            PyShape::new(Box::new(Rectangle::new(x.0, y.0, width.0, height.0))),
        )
    }

    #[staticmethod]
    fn fullscreen() -> PyShape {
        PyShape::new(Box::new(Rectangle::FULLSCREEN))
    }
}

#[pyclass(name = "Circle", extends = PyShape)]
pub struct PyCircle();

#[pymethods]
impl PyCircle {
    #[new]
    fn __new__(x: PySize, y: PySize, radius: PySize) -> (Self, PyShape) {
        (
            PyCircle(),
            PyShape::new(Box::new(Circle::new(x.0, y.0, radius.0))),
        )
    }
}

// Wrapper for the Stimulus trait
#[pyclass(name = "Stimulus", subclass)]
pub struct PyStimulus();

// The VideoStimulus
#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
#[pyclass(name = "VideoStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyVideoStimulus(VideoStimulus);

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
#[pymethods]
impl PyVideoStimulus {
    #[new]
    fn __new__(
        window: &PyWindow,
        shape: &PyShape,
        path: &str,
        width: u32,
        height: u32,
        thumbnail: Option<f32>,
        init: Option<bool>,
    ) -> (Self, PyStimulus) {
        let stim = psychophysics::visual::stimuli::VideoStimulus::new(
            &window.0,
            shape.0.clone_box(),
            path,
            width,
            height,
            thumbnail,
            init,
        );
        (PyVideoStimulus(stim), PyStimulus())
    }

    fn init(&mut self) {
        self.0.init().unwrap()
    }

    fn play(&mut self) {
        self.0.play().unwrap()
    }

    fn pause(&mut self) {
        self.0.pause().unwrap()
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
impl Stimulus for PyVideoStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        self.0.prepare(window, window_state, gpu_state)
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.0.render(enc, view)
    }
}

// ImageStimulus
#[pyclass(name = "ImageStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyImageStimulus(ImageStimulus);

#[pymethods]
impl PyImageStimulus {
    #[new]
    fn __new__(
        window: &PyWindow,
        shape: &PyShape,
        path: &str,
        py: Python<'_>,
    ) -> (Self, PyStimulus) {
        let _self_wrapper = SendWrapper::new(window);
        py.allow_threads(move || {
            let stim = ImageStimulus::new(&window.0, shape.0.clone_box(), path);
            (PyImageStimulus(stim), PyStimulus())
        })
    }

    fn set_translation(&mut self, x: PySize, y: PySize) {
        self.0.set_translation(x.0, y.0)
    }
}

impl Stimulus for PyImageStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        self.0.prepare(window, window_state, gpu_state)
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.0.render(enc, view)
    }
}

// SpriteStimulus
#[pyclass(name = "SpriteStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PySpriteStimulus(SpriteStimulus);

#[pymethods]
impl PySpriteStimulus {
    #[new]
    fn __new__(
        window: &PyWindow,
        shape: &PyShape,
        sprite_path: &str,
        num_sprites_x: u32,
        num_sprites_y: u32,
        fps: Option<f64>,
        repeat: Option<u64>,
    ) -> (Self, PyStimulus) {
        let stim = SpriteStimulus::new_from_spritesheet(
            &window.0,
            shape.0.clone_box(),
            sprite_path,
            num_sprites_x,
            num_sprites_y,
            fps,
            repeat,
        );
        (PySpriteStimulus(stim), PyStimulus())
    }

    fn advance_image_index(&mut self) {
        self.0.advance_image_index()
    }

    fn reset(&mut self) {
        self.0.reset()
    }

    fn set_translation(&mut self, x: PySize, y: PySize) {
        self.0.set_translation(x, y)
    }
}

impl Stimulus for PySpriteStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        self.0.prepare(window, window_state, gpu_state)
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.0.render(enc, view)
    }
}

// GaborStimulus
#[pyclass(name = "GaborStimulus", extends = PyStimulus)]
#[derive(Clone)]
pub struct PyGaborStimulus(GaborStimulus);

#[pymethods]
impl PyGaborStimulus {
    #[new]
    fn __new__(
        window: &PyWindow,
        shape: &PyShape,
        phase: f32,
        cycle_length: PySize,
        std_x: PySize,
        std_y: PySize,
        orientation: f32,
        color: (f32, f32, f32),
        py: Python<'_>,
    ) -> (Self, PyStimulus) {
        let _self_wrapper = SendWrapper::new(window);
        py.allow_threads(move || {
            let stim = GaborStimulus::new(
                &window.0,
                shape.0.clone_box(),
                phase,
                cycle_length.0.clone(),
                std_x.0.clone(),
                std_y.0.clone(),
                orientation,
                psychophysics::visual::color::SRGBA::new(color.0, color.1, color.2, 1.0),
            );
            (PyGaborStimulus(stim), PyStimulus())
        })
    }

    fn set_phase(&mut self, phase: f32) {
        self.0.set_phase(phase)
    }

    fn phase(&self) -> f32 {
        self.0.phase()
    }

    fn set_cycle_length(&mut self, cycle_length: PySize) {
        self.0.set_cycle_length(cycle_length.0)
    }

    fn cycle_length(&self) -> PySize {
        PySize(self.0.cycle_length())
    }

    fn set_color(&mut self, color: (f32, f32, f32)) {
        self.0.set_color(psychophysics::visual::color::SRGBA::new(
            color.0, color.1, color.2, 1.0,
        ))
    }

    fn color(&self) -> (f32, f32, f32) {
        let color = self.0.color();
        (color.r, color.g, color.b)
    }

    fn set_orientation(&mut self, orientation: f32) {
        self.0.set_orientation(orientation)
    }

    fn orientation(&self) -> f32 {
        self.0.orientation()
    }

    fn translate(&mut self, x: PySize, y: PySize) {
        self.0.translate(x, y)
    }

    fn set_translation(&mut self, x: PySize, y: PySize) {
        self.0.set_translation(x, y)
    }
}

impl Stimulus for PyGaborStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        self.0.prepare(window, window_state, gpu_state)
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        self.0.render(enc, view)
    }
}

// // The TextStimulus
// #[pyclass(name = "TextStimulus", extends = PyStimulus)]
// #[derive(Clone)]
// pub struct PyTextStimulus(TextStimulus);

// #[pymethods]
// impl PyTextStimulus {
//     #[new]
//     fn __new__(window: &PyWindow, text: &str) -> (Self, PyStimulus) {
//         let stim = TextStimulus::new(&window.0, text, rect.0);
//         (PyTextStimulus(stim), PyStimulus())
//     }
// }

// Sizes
// #[pyclass(name = "Size", subclass)]
// pub struct PySize(Size);

py_wrap!(Size, subclass);

impl Clone for PySize {
    fn clone(&self) -> Self {
        PySize(self.0.clone())
    }
}

// // implement Into<Size> for PySize
// impl Into<Size> for PySize {
//     fn into(self) -> Size {
//         self.0
//     }
// }

// Pixels
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

#[pymethods]
impl PyEvent {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    #[getter]
    fn timestamp(&self) -> f64 {
        // timestamo is an Instant, convert to f64 (seconds since epoch)
        self.0
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64()
    }

    #[getter]
    fn data(&self) -> PyEventData {
        let event = self.0.data.clone();
        match event {
            EventData::KeyPress { key, code } => PyEventData::KeyPress { key, code },
            EventData::KeyRelease { key, code } => PyEventData::KeyRelease { key, code },
            EventData::MouseButtonPress { button, position } => {
                PyEventData::MouseButtonPress {
                    button: PyMouseButton::from(button),
                    position: (PySize(position.0), PySize(position.1)),
                }
            }
            EventData::CursorMoved { position } => PyEventData::CursorMoved {
                position: (PySize(position.0), PySize(position.1)),
            },
            EventData::Other(desc) => PyEventData::Other { desc },
            // this should never happen
            _ => PyEventData::Other {
                desc: "Invalid event data encountered when converting to PyEventData"
                    .to_string(),
            },
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct MyClass {
    my_field: Box<i32>,
}

// #[pyo3::prelude::pyclass(name = "MouseButton")]
// #[derive(Debug, Clone)]
// pub struct PyMouseButton(pub MouseButton);

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
    KeyPress {
        key: String,
        code: u32,
    },
    KeyRelease {
        key: String,
        code: u32,
    },
    MouseButtonPress {
        button: PyMouseButton,
        position: (PySize, PySize),
    },
    CursorMoved {
        position: (PySize, PySize),
    },
    Other {
        desc: String,
    },
}

// Handling for user input

#[pymodule]
fn psychophysics_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    //pyo3::prepare_freethreaded_python();

    m.add_class::<PyExperimentManager>()?;
    m.add_class::<PyMonitor>()?;
    m.add_class::<PyWindowOptions>()?;
    m.add_class::<PyWindow>()?;
    m.add_class::<PyWindowManager>()?;
    m.add_class::<PyFrame>()?;
    m.add_class::<PyShape>()?;
    m.add_class::<PyRectangle>()?;
    m.add_class::<PyCircle>()?;

    m.add_class::<PyGaborStimulus>()?;
    m.add_class::<PyImageStimulus>()?;
    m.add_class::<PySpriteStimulus>()?;

    m.add_class::<PyStimulus>()?;
    m.add_class::<PySize>()?;
    m.add_class::<PyPixels>()?;
    m.add_class::<PyScreenWidth>()?;
    m.add_class::<PyScreenHeight>()?;

    m.add_class::<PyEventReceiver>()?;
    m.add_class::<PyEventVec>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyEventData>()?;

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
    m.add_class::<PyVideoStimulus>()?;

    Ok(())
}
