use pyo3::pyclass;

#[derive(Debug, Clone)]
#[pyclass]
pub enum Fill {
    Solid 
}