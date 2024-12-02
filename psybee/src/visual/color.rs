use pyo3::prelude::*;
use csscolorparser;
use crate::visual::geometry::IntoSize;

#[pyclass(name = "LinRgba")]
#[derive(Debug, Clone, Copy)]
pub struct LinRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl LinRgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    fn srgb_to_lin_rgb(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn from_srgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        // Decode sRGB to linear RGB
        Self {
            r: Self::srgb_to_lin_rgb(r),
            g: Self::srgb_to_lin_rgb(g),
            b: Self::srgb_to_lin_rgb(b),
            a,
        }

    }

    pub fn from_str(css_color_str: &str) -> Self {
        let color = csscolorparser::parse(css_color_str).expect("Failed to parse color");
        Self {
            r: Self::srgb_to_lin_rgb(color.r),
            g: Self::srgb_to_lin_rgb(color.g),
            b: Self::srgb_to_lin_rgb(color.b),
            a: color.a,
        }
    }

}

impl From<LinRgba> for renderer::prelude::RGBA {
    fn from(rgba: LinRgba) -> Self {
        Self {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            a: rgba.a,
        }
    }
}

#[pymethods]
impl LinRgba {
    #[new]
    #[pyo3(signature = (r, g, b, a = 1.0))]
    fn __new__(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    #[staticmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(css_color_str: &str) -> Self {
        Self::from_str(css_color_str)
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

#[derive(Debug, Clone, Copy)]
pub struct IntoLinRgba(pub LinRgba);

impl IntoLinRgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(LinRgba::new(r, g, b, a))
    }
}

impl From<IntoLinRgba> for LinRgba {
    fn from(into_lin_rgba: IntoLinRgba) -> Self {
        into_lin_rgba.0
    }
}



impl<'py> FromPyObject<'py> for IntoLinRgba {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // try to extract a LinRgba from the object
        if let Ok(rgba) = ob.extract::<LinRgba>() {
            Ok(Self(rgba))
        }
        // try to extract a tuple of 3 (alpha implicitly set to 1.0)
        else if let Ok((r, g, b)) = ob.extract() {
            Ok(Self(LinRgba::new(r, g, b, 1.0)))
        }
        // try to extract a tuple of 4
        else if let Ok((r, g, b, a)) = ob.extract() {
            Ok(Self(LinRgba::new(r, g, b, a)))
        }
        // try to extract from a string
        else if let Ok(css_color_str) = ob.extract::<&str>() {
            Ok(Self(LinRgba::from_str(css_color_str)))
        }
        // otherwise, raise an error
        else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Expected a tuple of 3 or 4 floats, a LinRgba, or a CSS color string",
            ))
        }
    }
}