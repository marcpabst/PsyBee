use psychophysics::{
    errors,
    visual::{geometry::Rectangle, window::Frame, Window},
    ExperimentManager, Monitor, WindowOptions,
};
use pyo3::{prelude::*, types::PyFunction, Python};

use send_wrapper::SendWrapper;

#[pyclass(unsendable, name = "ExperimentManager")]
pub struct PyExperimentManager(ExperimentManager);

#[pymethods]
impl PyExperimentManager {
    #[new]
    fn __new__() -> Self {
        PyExperimentManager(ExperimentManager::new())
    }

    fn __str__(&self) -> String {
        format!("ExperimentManager")
    }

    fn __repr__(&self) -> String {
        // return the Debug representation of the ExperimentManager
        format!("{:?}", self.0)
    }

    fn get_available_monitors(&self) -> Vec<PyMonitor> {
        self.0
            .get_available_monitors()
            .iter()
            .map(|m| PyMonitor(m.clone()))
            .collect()
    }

    fn run_experiment(
        &mut self,
        py: Python,
        window_options: &PyWindowOptions,
        experiment_fn: Py<PyFunction>,
    ) {
        let rust_experiment_fn = move |window: Window| -> Result<
            (),
            errors::PsychophysicsError,
        > {
            Python::with_gil(|py| -> PyResult<()> {
                let pywin = PyWindow(window);
                experiment_fn.call1(py, (pywin,)).unwrap();
                Ok(())
            })
            .unwrap();
            Ok(())
        };

        // wrap self in a SendWrapper so that it can be sent through the magic barrier
        let mut self_wrapper = SendWrapper::new(self);

        py.allow_threads(move || {
            self_wrapper
                .0
                .run_experiment(&window_options.0, rust_experiment_fn)
        });
    }
}

#[pyclass(unsendable, name = "Monitor")]
pub struct PyMonitor(Monitor);

#[pymethods]
impl PyMonitor {
    fn __repr__(&self) -> String {
        // return the Debug representation of the Monitor
        format!("{:?}", self.0)
    }
}

#[pyclass(unsendable, name = "Window")]
#[derive(Debug)]
pub struct PyWindow(Window);

#[pymethods]
impl PyWindow {
    fn get_frame(&self) -> PyFrame {
        PyFrame(self.0.get_frame())
    }

    fn submit_frame(&mut self, frame: &PyFrame) {
        self.0.submit_frame(frame.0.clone());
    }

    fn __repr__(&self) -> String {
        // return the Debug representation of the Window
        format!("{:?}", self.0)
    }
}

#[pyclass(unsendable, name = "Frame")]
pub struct PyFrame(Frame);

#[pymethods]
impl PyFrame {
    fn set_bg_color(&mut self, color: (f32, f32, f32)) {
        self.0.set_bg_color(
            psychophysics::visual::color::SRGBA::new(
                color.0, color.1, color.2, 1.0,
            ),
        );
    }

    fn add(&mut self, stimulus: &PyShapeStimulus) {
        self.0.add(&stimulus.0);
    }
}

/// An object that contains the options for a window.
#[pyclass(name = "WindowOptions")]
pub struct PyWindowOptions(WindowOptions);

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
            "windowed" => PyWindowOptions(WindowOptions::Windowed {
                resolution: None,
            }),
            "fullscreen_exact" => {
                PyWindowOptions(WindowOptions::FullscreenExact {
                    resolution: resolution,
                    monitor: monitor.map(|m| m.0.clone()),
                    refresh_rate: refresh_rate,
                })
            }
            "fullscreen_highest_resolution" => PyWindowOptions(
                WindowOptions::FullscreenHighestResolution {
                    monitor: monitor.map(|m| m.0.clone()),
                    refresh_rate: refresh_rate,
                },
            ),
            "fullscreen_highest_refresh_rate" => PyWindowOptions(
                WindowOptions::FullscreenHighestRefreshRate {
                    monitor: monitor.map(|m| m.0.clone()),
                    resolution: resolution,
                },
            ),
            _ => panic!("Unknown mode: {}", mode),
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

use psychophysics::visual::stimuli::ShapeStimulus;
#[pyclass(unsendable, name = "ShapeStimulus")]
pub struct PyShapeStimulus(ShapeStimulus);

#[pymethods]
impl PyShapeStimulus {
    #[new]
    fn __new__(window: &PyWindow, color: (f32, f32, f32)) -> Self {
        PyShapeStimulus(ShapeStimulus::new(
            &window.0,
            Rectangle::FULLSCREEN,
            psychophysics::visual::color::SRGBA::new(
                color.0, color.1, color.2, 1.0,
            ),
        ))
    }

    fn set_color(&mut self, color: (f32, f32, f32)) {
        self.0.set_color(psychophysics::visual::color::SRGBA::new(
            color.0, color.1, color.2, 1.0,
        ));
    }
}

#[pymodule]
fn psychophysics_py<'py, 'a>(
    _py: Python<'py>,
    m: &'a pyo3::prelude::PyModule,
) -> Result<(), pyo3::PyErr> {
    m.add_class::<PyExperimentManager>()?;
    m.add_class::<PyMonitor>()?;
    m.add_class::<PyWindowOptions>()?;
    m.add_class::<PyWindow>()?;
    m.add_class::<PyShapeStimulus>()?;
    m.add_class::<PyFrame>()?;
    Ok(())
}
