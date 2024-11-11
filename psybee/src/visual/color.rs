use pyo3::prelude::*;

#[pyclass(name = "Rgba")]
#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Rgba> for renderer::prelude::RGBA {
    fn from(rgba: Rgba) -> Self {
        Self {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            a: rgba.a,
        }
    }
}

#[pymethods]
impl Rgba {
    #[new]
    #[pyo3(signature = (r, g, b, a))]
    fn __new__(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[getter]
    fn r(&self) -> f32 {
        self.r
    }

    #[getter]
    fn g(&self) -> f32 {
        self.g
    }

    #[getter]
    fn b(&self) -> f32 {
        self.b
    }

    #[getter]
    fn a(&self) -> f32 {
        self.a
    }
}
