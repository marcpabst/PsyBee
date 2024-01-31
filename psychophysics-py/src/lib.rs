use psychophysics::{
    errors, visual::Window, ExperimentManager, Monitor, WindowOptions,
};
use pyo3::{prelude::*, types::PyFunction, Python};

#[pyclass(unsendable, name = "ExperimentManager")]
pub struct PyExperimentManager(ExperimentManager);

#[derive(Debug)]
struct ThreadSafePyFunction(PyFunction);

unsafe impl Send for ThreadSafePyFunction {}
unsafe impl Sync for ThreadSafePyFunction {}

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
        window_options: &PyWindowOptions,
        experiment_fn: &PyFunction,
    ) {
        // create rust FnOnce(Window) -> Result<(), errors::PsychophysicsError> + 'static + Send,
        let rust_experiment_fn = move |window: Window| -> Result<
            (),
            errors::PsychophysicsError,
        > {
            experiment_fn.call0();
            Ok(())
        };

        // allow python function to be called from another thread

        // run the experiment
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

#[pymodule]
fn psychophysics_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyExperimentManager>()?;
    m.add_class::<PyMonitor>()?;
    m.add_class::<PyWindowOptions>()?;
    Ok(())
}
