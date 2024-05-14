use pyo3::prelude::*;
use rand::Rng;
use std::cmp::Ordering;
use std::io;

use pyo3::types::PyFunction;

/// Function that takes a Python callable and calls it after 10s from a separate thread.
#[pyfunction]
fn add_event_handler(py: Python<'_>, handler: Py<PyFunction>) -> PyResult<()> {
    py.allow_threads(move || {
        // Call the Python function after 10s.
        run_delayed(1, move |u| {
            Python::with_gil(|py| {
                handler.call1(py, (u,)).unwrap();
            });
        });
    });

    Ok(())
}

/// Runs a function after a delay.
fn run_delayed<F>(delay: u64, f: F)
where
    F: (Fn(f64) -> ()) + Send + 'static,
{
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(delay));
        f(42.0);
    });
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn ptest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add_event_handler, m)?)?;
    Ok(())
}
