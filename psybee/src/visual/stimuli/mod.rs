use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use animations::{Animation, Repeat, TransitionFunction};
use numpy::PyUntypedArrayMethods;
#[macro_use]
use uuid::Uuid;

use dyn_clone::DynClone;

use super::{
    color::Rgba,
    geometry::{IntoSize, Size, Transformation2D},
    window::Window,
    window::WrappedWindow,
};

use pyo3::{exceptions::PyValueError, prelude::*};

use renderer::{image::GenericImageView, VelloScene};

pub mod animations;
pub mod gabor;
pub mod image;
pub mod vector;
pub mod sprite;
pub mod text;
pub mod grid;
pub mod shape;

pub type WrappedStimulus = Arc<Mutex<dyn Stimulus>>;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum StimulusParamValue {
    Size(Size),
    f64(f64),
    String(String),
    bool(bool),
    i64(i64),
    Rgba(Rgba),
}

macro_rules! is_variant {
    ($value:expr, $pattern:path) => {
        matches!($value, $pattern { .. } | $pattern)
    };
}

impl StimulusParamValue {
    fn is_f64(&self) -> bool {
        match self {
            StimulusParamValue::f64(_) => true,
            _ => false,
        }
    }
}

pub struct IntoStimulusParamValue(pub StimulusParamValue);

impl From<IntoStimulusParamValue> for StimulusParamValue {
    fn from(value: IntoStimulusParamValue) -> Self {
        value.0
    }
}

impl<'py> FromPyObject<'py> for IntoStimulusParamValue {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = ob.extract::<f64>() {
            return Ok(Self(StimulusParamValue::f64(value)));
        }
        if let Ok(value) = ob.extract::<String>() {
            return Ok(Self(StimulusParamValue::String(value)));
        }
        if let Ok(value) = ob.extract::<bool>() {
            return Ok(Self(StimulusParamValue::bool(value)));
        }
        if let Ok(value) = ob.extract::<i64>() {
            return Ok(Self(StimulusParamValue::i64(value)));
        }
        if let Ok(value) = ob.extract::<Rgba>() {
            return Ok(Self(StimulusParamValue::Rgba(value)));
        }
        if let Ok(value) = ob.extract::<Size>() {
            return Ok(Self(StimulusParamValue::Size(value)));
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Could not convert the value to a StimulusParamValue",
        ))
    }
}

pub trait StimulusParams {
    fn get_param(&self, name: &str) -> Option<StimulusParamValue>;
    fn set_param(&mut self, name: &str, value: StimulusParamValue);
}

/// The stimulus trait.
pub trait Stimulus: Send + Sync + downcast_rs::Downcast + std::fmt::Debug {
    /// Draw the stimulus onto the scene.
    fn draw(&mut self, scene: &mut VelloScene, window: &Window);

    /// Check if the stimulus contains a specific Point.
    fn contains(&self, x: Size, y: Size, window: &WrappedWindow) -> bool {
        // by default, stimuli will report false for contains
        false
    }

    /// Return the UUID that identifies the stimulus.
    fn uuid(&self) -> Uuid;

    /// Check if two stimuli are equal.
    fn equal(&self, other: &dyn Stimulus) -> bool {
        self.uuid() == other.uuid()
    }

    /// Returns true if the stimulus is currently visible.
    fn visible(&self) -> bool {
        true
    }

    /// Set the visibility of the stimulus.
    fn set_visible(&mut self, visible: bool) {
        // do nothing by default
    }

    /// Hide the stimulus. This is a convenience method that calls
    /// `set_visible(false)`.
    fn hide(&mut self) -> () {
        self.set_visible(false);
    }

    /// Show the stimulus. This is a convenience method that calls
    fn show(&mut self) -> () {
        self.set_visible(true);
    }

    /// Toggle the visibility of the stimulus.
    fn toggle_visibility(&mut self) -> () {
        self.set_visible(!self.visible());
    }

    // Animation methods

    /// Returns the animations that are associated with this stimulus.
    fn animations(&mut self) -> &mut Vec<Animation> {
        panic!("animations not implemented for this stimulus");
    }

    /// Add an animation to the object.
    fn add_animation(&mut self, animation: Animation) {
        // do nothing by default
    }

    /// Animate a specific attribute of the object.
    fn animate(
        &mut self,
        parameter: &str,
        from: StimulusParamValue,
        to: StimulusParamValue,
        duration: f64,
        repeat: Repeat,
        easing: TransitionFunction,
    ) {
        let animation = Animation::new(parameter, from, to, duration, Instant::now(), repeat, easing);
        self.add_animation(animation);
    }

    /// Update the object's state based on the current time. Finished animations are removed.
    fn update_animations(&mut self, time: Instant, window: &Window) {
        let mut params_to_set = Vec::new();

        self.animations().retain_mut(|animation| {
            if animation.finished(time) {
                return false;
            }
            let value = animation.value(time, window);
            params_to_set.push((animation.parameter().to_string(), value));
            true
        });

        for (param, value) in params_to_set {
            self.set_param(&param, value);
        }
    }

    /// Set the transformation.
    fn set_transformation(&mut self, transformation: Transformation2D) -> ();

    /// Add a transformation to the current transformation.
    fn add_transformation(&mut self, transformation: Transformation2D) {
        self.set_transformation(transformation * self.transformation());
    }

    /// Translate the object by the given x and y coordinates.
    fn translate(&mut self, x: Size, y: Size) {
        self.add_transformation(Transformation2D::Translation(x, y));
    }

    /// Scale the object by the given x and y factors.
    fn scale_point(&mut self, sx: f32, sy: f32, x: Size, y: Size) {
        self.add_transformation(Transformation2D::ScalePoint(sx, sy, x, y));
    }

    /// Set the translation of the object to the given x and y coordinates. This
    /// overwrites any previously applied transformations.
    fn set_translation(&mut self, x: Size, y: Size) {
        self.set_transformation(Transformation2D::Translation(x, y));
    }

    /// Return the current transformation.
    fn transformation(&self) -> Transformation2D;

    /// Transforms a point from the window coordinate system to the stimulus.
    /// coordinate system.
    fn transform_point(&self, x: f32, y: f32, window: &Window) -> (f32, f32) {
        self.transformation().transform_point(x, y, &window.physical_properties)
    }

    /// Rotate the object around the center of the object by the given angle.
    fn rotate_center(&mut self, angle: f32) {
        self.set_transformation(Transformation2D::RotationCenter(angle));
    }

    /// Rotate the object around the given point by the given angle.
    fn rotate_point(&mut self, angle: f32, x: Size, y: Size) {
        self.set_transformation(Transformation2D::RotationPoint(angle, x, y));
    }

    /// Scale the object around the center of the object by the given x and y
    /// factors.
    fn scale_center(&mut self, x: f32, y: f32) {
        self.set_transformation(Transformation2D::ScaleCenter(x, y));
    }

    /// Shear the object around the center of the object by the given x and y
    /// factors.
    fn shear_center(&mut self, x: f32, y: f32) {
        self.set_transformation(Transformation2D::ShearCenter(x, y));
    }

    /// Shear the object around the given point by the given x and y factors.
    fn shear_point(&mut self, x: f32, y: f32, x0: Size, y0: Size) {
        self.set_transformation(Transformation2D::ShearPoint(x, y, x0, y0));
    }

    /// Get a parameter of the stimulus.
    fn get_param(&self, name: &str) -> Option<StimulusParamValue>;

    /// Set a parameter of the stimulus.
    fn set_param(&mut self, name: &str, value: StimulusParamValue);
}

downcast_rs::impl_downcast!(Stimulus);

/// Wraps a Stimulus. This class is used either as a base class for other
/// stimulus classes or as a standalone class, when no specific runtume type
/// information is available.
#[pyclass(name = "Stimulus", subclass)]
#[derive(Debug, Clone)]
pub struct PyStimulus(pub WrappedStimulus);

macro_rules! downcast_stimulus {
    ($slf:ident, $name:ident) => {
        $slf.as_super()
            .0
            .lock()
            .unwrap()
            .downcast_ref::<$name>()
            .expect("downcast failed")
    };
}

macro_rules! downcast_stimulus_mut {
    ($slf:ident, $name:ident) => {
        $slf.as_super()
            .0
            .lock()
            .unwrap()
            .downcast_mut::<$name>()
            .expect("downcast failed")
    };
}

// macro that implements pyo3 methods for a warapper Py$name
macro_rules! impl_pystimulus_for_wrapper {
    ($wrapper:ident, $name:ident) => {
        use crate::visual::geometry::IntoSize;
        use crate::visual::stimuli::downcast_stimulus;
        use crate::visual::stimuli::downcast_stimulus_mut;
        use crate::visual::stimuli::IntoStimulusParamValue;
        use crate::visual::stimuli::Repeat;
        use crate::visual::stimuli::TransitionFunction;
        use crate::visual::window::WrappedWindow;
        use std::mem;

        #[pymethods]
        impl $wrapper {
            fn __getitem__(mut slf: PyRef<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
                let py = slf.py();

                let param = downcast_stimulus!(slf, $name).get_param(name);

                // extract the value from the StimulusParam
                match param {
                    Some(StimulusParamValue::Size(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::f64(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::String(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::bool(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::i64(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::Rgba(val)) => Ok(val.into_py(py)),
                    None => Err(PyValueError::new_err("parameter not found")),
                }
            }

            fn __setitem__(mut slf: PyRefMut<'_, Self>, name: &str, value: Py<PyAny>) -> PyResult<()> {
                let py = slf.py();

                let current_val = &downcast_stimulus!(slf, $name).get_param(name).unwrap();

                match current_val {
                    StimulusParamValue::String(_) => {
                        let value = value.extract::<String>(py)?;
                        downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::String(value));
                        return Ok(());
                    }
                    StimulusParamValue::Size(_) => {
                        let value = value.extract::<IntoSize>(py)?;
                        downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::Size(value.into()));
                        return Ok(());
                    }
                    StimulusParamValue::f64(_) => {
                        let value = value.extract::<f64>(py)?;
                        downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::f64(value));
                        return Ok(());
                    }
                    StimulusParamValue::bool(_) => {
                        let value = value.extract::<bool>(py)?;
                        downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::bool(value));
                        return Ok(());
                    }
                    StimulusParamValue::i64(_) => {
                        let value = value.extract::<i64>(py)?;
                        downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::i64(value));
                        return Ok(());
                    }
                    // StimulusParamValue::Rgba(_) => {
                    //     let value = value.extract::<Rgba>(py)?;
                    //     downcast_stimulus_mut!(slf, $name).set_param(name, StimulusParamValue::Rgba(value));
                    //     return Ok(());
                    // }
                    _ => {}
                }

                return Err(PyValueError::new_err("parameter not found"));
            }

            /// Rotate the stimulus at a given point.
            fn rotated_at(mut slf: PyRefMut<'_, Self>, angle: f32, x: IntoSize, y: IntoSize) -> PyRefMut<'_, Self> {
                downcast_stimulus_mut!(slf, $name).rotate_point(angle, x.into(), y.into());
                slf
            }

            /// Translate the stimulus.
            fn translated<'a>(mut slf: PyRefMut<'a, Self>, x: IntoSize, y: IntoSize) -> PyRefMut<'a, Self> {
                downcast_stimulus_mut!(slf, $name).translate(x.into(), y.into());
                slf
            }

            /// Scale the stimulus from a given point
            fn scaled_at<'a>(
                mut slf: PyRefMut<'a, Self>,
                sx: f32,
                sy: f32,
                x: IntoSize,
                y: IntoSize,
            ) -> PyRefMut<'a, Self> {
                downcast_stimulus_mut!(slf, $name).scale_point(sx, sy, x.into(), y.into());
                slf
            }

            // Hide the stimulus
            fn hide(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
                downcast_stimulus_mut!(slf, $name).hide();
                slf
            }

            // Show the stimulus
            fn show(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
                downcast_stimulus_mut!(slf, $name).show();
                slf
            }

            // Return current visibility
            fn visible(mut slf: PyRefMut<'_, Self>) -> bool {
                downcast_stimulus!(slf, $name).visible()
            }

            fn contains(mut slf: PyRefMut<'_, Self>, x: IntoSize, y: IntoSize, window: &WrappedWindow) -> bool {
                downcast_stimulus!(slf, $name).contains(x.into(), y.into(), window)
            }

            // Animate the stimulus
            fn animate(mut slf: PyRefMut<'_, Self>, param_name: &str, to: Py<PyAny>, duration: f64) -> PyResult<()> {
                let from = downcast_stimulus!(slf, $name)
                    .get_param(param_name)
                    .ok_or_else(|| PyValueError::new_err(format!("parameter {} not found", param_name)))?;

                // extract `to` value with the correct type
                let to = match from {
                    StimulusParamValue::Size(_) => {
                        StimulusParamValue::Size(to.extract::<IntoSize>(slf.py()).expect("invalid value").into())
                    }
                    StimulusParamValue::f64(_) => {
                        StimulusParamValue::f64(to.extract::<f64>(slf.py()).expect("invalid value"))
                    }
                    StimulusParamValue::String(_) => {
                        StimulusParamValue::String(to.extract::<String>(slf.py()).expect("invalid value"))
                    }
                    StimulusParamValue::bool(_) => {
                        StimulusParamValue::bool(to.extract::<bool>(slf.py()).expect("invalid value"))
                    }
                    StimulusParamValue::i64(_) => {
                        StimulusParamValue::i64(to.extract::<i64>(slf.py()).expect("invalid value"))
                    }
                    _ => return Err(PyValueError::new_err("invalid value type")),
                };

                downcast_stimulus_mut!(slf, $name).animate(
                    param_name,
                    from.into(),
                    to.into(),
                    duration,
                    Repeat::Loop(20),
                    TransitionFunction::None,
                );
                Ok(())
            }
        }
    };
}

#[derive(Debug, Clone)]
#[pyclass]
#[pyo3(name = "Image")]
pub struct WrappedImage(Arc<Mutex<renderer::brushes::Image>>);

impl WrappedImage {
    /// Create a new WrappedImage from a DynamicImage.
    pub fn from_dynamic_image(image: renderer::image::DynamicImage) -> Self {
        let vello_image = renderer::brushes::Image::new(&image);
        WrappedImage(Arc::new(Mutex::new(vello_image)))
    }

    /// Create a new WrappedImage from a file path.
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, renderer::image::ImageError> {
        let image = renderer::image::open(path)?;
        Ok(Self::from_dynamic_image(image))
    }

    pub fn inner(&self) -> std::sync::MutexGuard<renderer::brushes::Image> {
        self.0.lock().unwrap()
    }

    pub fn to_gpu(&mut self, window: &WrappedWindow) {
        let mut image = self.inner();
        let win = window.inner();
        let gpu_state = win.gpu_state.lock().unwrap();
        let device = &gpu_state.device;
        let queue = &gpu_state.queue;
        image.to_gpu(&device, &queue);
    }

    pub fn from_spritesheet_path(src: String, nx: u32, ny: u32) -> Vec<Self> {
        let image = renderer::image::open(src).expect("Failed to load image");
        // split the image into nx * ny sprites
        let (w, h) = (image.width() / nx as u32, image.height() / ny as u32);
        // make sure the image is divisible by nx and ny
        assert_eq!(image.width() % nx as u32, 0, "Image width is not divisible by nx");
        assert_eq!(image.height() % ny as u32, 0, "Image height is not divisible by ny");

        let mut images = Vec::new();
        for y in 0..ny {
            for x in 0..nx {
                let sprite = image.view(x * w, y * h, w, h).to_image();
                images.push(WrappedImage::from_dynamic_image(
                    renderer::image::DynamicImage::ImageRgba8(sprite),
                ));
            }
        }
        images
    }
}

#[pymethods]
impl WrappedImage {
    #[new]
    fn __new__(path: &str) -> PyResult<Self> {
        Self::from_path(path).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Create a new a List of WrappedImage from a spritesheet.
    #[staticmethod]
    #[pyo3(name = "from_spritesheet")]
    pub fn py_from_spritesheet(src: &str, nx: u32, ny: u32) -> PyResult<Vec<Self>> {
        let images = WrappedImage::from_spritesheet_path(src.to_string(), nx, ny);
        Ok(images)
    }

    /// Create a new WrappedImage from a 2D numpy array.
    #[staticmethod]
    #[pyo3(name = "from_numpy")]
    pub fn py_from_numpy(array: numpy::PyArrayLike<f32, numpy::Ix3, numpy::AllowTypeChange>) -> Self {
        let array = array.as_array();
        let image = renderer::image::DynamicImage::ImageRgb8(
            renderer::image::RgbImage::from_raw(
                array.shape()[1] as u32,
                array.shape()[0] as u32,
                array
                    .as_slice()
                    .expect("failed to get slice")
                    .iter()
                    .map(|&x| (x * 255.0) as u8)
                    .collect(),
            )
            .expect("failed to create image"),
        );
        WrappedImage::from_dynamic_image(image)
    }

    /// Move the image to the GPU.
    #[pyo3(name = "move_to_gpu")]
    fn py_to_gpu(&mut self, window: &WrappedWindow) -> Self {
        self.to_gpu(window);
        self.clone()
    }
}

pub(crate) use downcast_stimulus;
pub(crate) use downcast_stimulus_mut;
pub(crate) use impl_pystimulus_for_wrapper;
