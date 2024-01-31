use psychophysics::{
    errors,
    visual::{geometry::Rectangle, window::Frame, Window},
    ExperimentManager, Monitor, WindowOptions,
};
use pyo3::{prelude::*, types::PyFunction, Python};

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
        // create rust FnOnce(Window) -> Result<(), errors::PsychophysicsError> + 'static + Send,
        let rust_experiment_fn = move |window: Window| -> Result<
            (),
            errors::PsychophysicsError,
        > {
            println!("Calling python function");
            // check
            unsafe {
                let py = Python::assume_gil_acquired();
                // Python::with_gil(
                //     // create a new python function that takes a Window as argument
                //     |py| -> PyResult<()> {
                let pywin = PyWindow(window);
                // call the python function with the window as argument
                //experiment_fn.call1(py, (pywin,)).unwrap();
                println!("Got the interpreter");
                experiment_fn.call1(py, (pywin,)).unwrap();
                //         Ok(())
                //     },
                // )
                // .unwrap();
                println!("Returned from python function");
            }
            Ok(())
        };

        self.0.run_experiment(&window_options.0, rust_experiment_fn);
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

#[pyclass(name = "WindowOptions")]
pub struct PyWindowOptions(WindowOptions);

#[pymethods]
impl PyWindowOptions {
    #[new]
    /// Create a new WindowOptions object.
    fn __new__() -> Self {
        PyWindowOptions(WindowOptions::Windowed { resolution: None })
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
