use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

use animations::{Animation, Repeat, TransitionFunction};
use numpy::PyUntypedArrayMethods;
#[macro_use]
use uuid::Uuid;

use dyn_clone::DynClone;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::image::GenericImageView;
use strum_macros::{Display, EnumString};

use super::{
    geometry::{IntoSize, Size, Transformation2D},
    window::{Frame, Window, WindowState},
};
use crate::visual::color::LinRgba;

pub mod animations;
mod helpers;

pub mod gabor;
// pub mod grid;
pub mod image;
pub mod pattern;
pub mod shape;
// pub mod sprite;
pub mod text;
// pub mod vector;
// pub mod video;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum StimulusParamValue {
    Size(Size),
    f64(f64),
    String(String),
    bool(bool),
    i64(i64),
    LinRgba(LinRgba),
    Shape(super::geometry::Shape),
    StrokeStyle(StrokeStyle),
}

#[derive(Debug, Clone, EnumString, Display, Default)]
pub enum StrokeStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    DashDot,
    Dashes(Vec<f64>),
}

// implement IntoPy for StrokeStyle (by converting it to a string)
impl IntoPy<PyObject> for StrokeStyle {
    fn into_py(self, py: Python) -> PyObject {
        self.to_string().into_py(py)
    }
}

// implement FromPyObject for StrokeStyle (by parsing it from a string)
impl<'p> FromPyObject<'p> for StrokeStyle {
    fn extract_bound(ob: &Bound<'p, PyAny>) -> PyResult<Self> {
        let s = ob.extract::<String>()?;
        match TryFrom::try_from(s.as_str()) {
            Ok(style) => Ok(style),
            Err(_) => Err(PyValueError::new_err("invalid stroke style")),
        }
    }
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
        if let Ok(value) = ob.extract::<LinRgba>() {
            return Ok(Self(StimulusParamValue::LinRgba(value)));
        }
        if let Ok(value) = ob.extract::<Size>() {
            return Ok(Self(StimulusParamValue::Size(value)));
        }
        if let Ok(value) = ob.extract::<super::geometry::Shape>() {
            return Ok(Self(StimulusParamValue::Shape(value)));
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
pub trait Stimulus: downcast_rs::Downcast + std::fmt::Debug + Send {
    /// Draw the stimulus onto the frame.
    fn draw(&mut self, scene: &mut Frame);

    /// Check if the stimulus contains a specific Point.
    fn contains(&self, x: Size, y: Size, window: &Window) -> bool {
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
    fn hide(&mut self) {
        self.set_visible(false);
    }

    /// Show the stimulus. This is a convenience method that calls
    fn show(&mut self) {
        self.set_visible(true);
    }

    /// Toggle the visibility of the stimulus.
    fn toggle_visibility(&mut self) {
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
    fn update_animations(&mut self, time: Instant, window_state: &WindowState) {
        let mut params_to_set = Vec::new();

        self.animations().retain_mut(|animation| {
            let value = animation.value(time, window_state);
            params_to_set.push((animation.parameter().to_string(), value));
            if animation.finished(time) {
                return false;
            } else {
                true
            }
        });

        for (param, value) in params_to_set.iter() {
            self.set_param(param, value.clone());
        }
    }

    /// Set the transformation.
    fn set_transformation(&mut self, transformation: Transformation2D);

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
        // TODO
        // self.transformation().transform_point(x, y, &window.physical_properties)
        (x, y)
    }

    /// Rotate the object around the given point by the given angle.
    fn rotate_point(&mut self, angle: f32, x: Size, y: Size) {
        self.set_transformation(Transformation2D::RotationPoint(angle, x, y));
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

#[derive(Debug, Clone)]
pub struct DynamicStimulus(Arc<Mutex<dyn Stimulus>>);

/// Wraps a Stimulus. This class is used either as a base class for other
/// stimulus classes or as a standalone class, when no specific runtume type
/// information is available.
#[pyclass(name = "Stimulus", subclass, module = "psydk.visual.stimuli")]
#[derive(Debug, Clone)]
pub struct PyStimulus(DynamicStimulus);

impl DynamicStimulus {
    pub fn new(stimulus: impl Stimulus + 'static) -> Self {
        Self(Arc::new(Mutex::new(stimulus)))
    }

    pub fn lock(&self) -> MutexGuard<dyn Stimulus> {
        self.0.lock().unwrap()
    }
}

// #[pymethods]
// impl PyStimulus {

//     fn new() -> Self {
//         Self(DynamicStimulus::new(shape::ShapeStimulus::new(
//             super::geometry::Shape::Rectangle,
//             Size::new(0.0, 0.0),
//             Size::new(0.0, 0.0),
//             None,
//             None,
//             None,
//             None,
//             None,
//             Transformation2D::default(),
//         )))
//     }

// }

impl PyStimulus {
    pub fn new(stimulus: impl Stimulus + 'static) -> Self {
        Self(DynamicStimulus::new(stimulus))
    }

    pub fn as_super(&self) -> &DynamicStimulus {
        &self.0
    }
}

macro_rules! downcast_stimulus {
    ($slf:ident, $name:ident) => {
        $slf.as_super()
            .0
            .lock()
            .downcast_ref::<$name>()
            .expect("downcast failed")
    };
}

macro_rules! downcast_py_stimulus_mut {
    ($slf:ident, $name:ident) => {
        $slf.as_super()
            .0
            .lock()
            .downcast_mut::<$name>()
            .expect("downcast failed")
    };
}

macro_rules! downcast_stimulus_mut {
    ($slf:ident, $name:ident) => {
        $slf.0.lock().downcast_mut::<$name>().expect("downcast failed")
    };
}

// macro that implements pyo3 methods for a warapper Py$name
macro_rules! impl_pystimulus_for_wrapper {
    ($wrapper:ident, $name:ident) => {
        use std::mem;

        use pyo3::{exceptions::PyValueError, prelude::*};

        use crate::visual::{
            geometry::IntoSize,
            stimuli::{
                downcast_py_stimulus_mut, downcast_stimulus, downcast_stimulus_mut, IntoStimulusParamValue, Repeat,
                TransitionFunction,
            },
            window::Window,
        };

        #[pymethods]
        impl $wrapper {
            fn __getitem__(slf: PyRef<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
                let py = slf.py();

                let param = downcast_stimulus!(slf, $name).get_param(name);

                // extract the value from the StimulusParam
                match param {
                    Some(StimulusParamValue::Size(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::f64(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::String(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::bool(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::i64(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::LinRgba(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::Shape(val)) => Ok(val.into_py(py)),
                    Some(StimulusParamValue::StrokeStyle(val)) => Ok(val.into_py(py)),
                    None => Err(PyValueError::new_err("parameter not found")),
                }
            }

            fn __setitem__(slf: Bound<Self>, name: &str, value: Py<PyAny>) -> PyResult<()> {
                let py = slf.py();

                // get DynamicStimulus from the wrapper
                let dynamic_stimulus = slf.as_super().borrow().0.clone();

                let current_val = py.allow_threads(move || dynamic_stimulus.0.lock().unwrap().get_param(name).unwrap());

                let dynamic_stimulus = slf.as_super().borrow().0.clone();

                match current_val {
                    StimulusParamValue::String(_) => {
                        let value = value.extract::<String>(py)?;
                        let value = StimulusParamValue::String(value);

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    StimulusParamValue::Size(_) => {
                        let value = value.extract::<IntoSize>(py)?;
                        let value = StimulusParamValue::Size(value.into());

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    StimulusParamValue::f64(_) => {
                        let value = value.extract::<f64>(py)?;
                        let value = StimulusParamValue::f64(value);

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    StimulusParamValue::bool(_) => {
                        let value = value.extract::<bool>(py)?;
                        let value = StimulusParamValue::bool(value);

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    StimulusParamValue::i64(_) => {
                        let value = value.extract::<i64>(py)?;
                        let value = StimulusParamValue::i64(value);

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    StimulusParamValue::LinRgba(_) => {
                        let value = value.extract::<crate::visual::color::IntoLinRgba>(py)?;
                        let value = StimulusParamValue::LinRgba(value.into());

                        py.allow_threads(move || {
                            let mut ds = dynamic_stimulus.0.lock().unwrap();
                            let ds = ds.downcast_mut::<$name>().expect("downcast failed");
                            ds.set_param(name, value);
                        });

                        return Ok(());
                    }
                    _ => {}
                }

                return Err(PyValueError::new_err("parameter not found"));
            }

            /// Rotate the stimulus at a given point.
            fn rotated_at(mut slf: PyRefMut<'_, Self>, angle: f32, x: IntoSize, y: IntoSize) -> PyRefMut<'_, Self> {
                downcast_py_stimulus_mut!(slf, $name).rotate_point(angle, x.into(), y.into());
                slf
            }

            /// Translate the stimulus.
            fn translated(mut slf: PyRefMut<'_, Self>, x: IntoSize, y: IntoSize) -> PyRefMut<'_, Self> {
                downcast_py_stimulus_mut!(slf, $name).translate(x.into(), y.into());
                slf
            }

            /// Scale the stimulus from a given point
            fn scaled_at(
                mut slf: PyRefMut<'_, Self>,
                sx: f32,
                sy: f32,
                x: IntoSize,
                y: IntoSize,
            ) -> PyRefMut<'_, Self> {
                downcast_py_stimulus_mut!(slf, $name).scale_point(sx, sy, x.into(), y.into());
                slf
            }

            // Hide the stimulus
            fn hide(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
                downcast_py_stimulus_mut!(slf, $name).hide();
                slf
            }

            // Show the stimulus
            fn show(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
                downcast_py_stimulus_mut!(slf, $name).show();
                slf
            }

            // Return current visibility
            fn visible(mut slf: PyRefMut<'_, Self>) -> bool {
                downcast_stimulus!(slf, $name).visible()
            }

            fn contains(mut slf: PyRefMut<'_, Self>, x: IntoSize, y: IntoSize, window: &Window) -> bool {
                downcast_stimulus!(slf, $name).contains(x.into(), y.into(), window)
            }

            /// Animate a parameter of the stimulus.
            /// The parameter must be a valid parameter of the stimulus.
            ///
            /// Parameters
            /// ----------
            /// param_name : str
            ///    The name of the parameter to animate.
            /// to :
            ///   The target value of the animation.
            /// duration : float
            ///  The duration of the animation in seconds.
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
                    _ => return Err(PyValueError::new_err("invalid value type for animation")),
                };

                downcast_py_stimulus_mut!(slf, $name).animate(
                    param_name,
                    from.into(),
                    to.into(),
                    duration,
                    Repeat::Loop(1),
                    TransitionFunction::None,
                );
                Ok(())
            }
        }
    };
}

pub(crate) use downcast_py_stimulus_mut;
pub(crate) use downcast_stimulus;
pub(crate) use downcast_stimulus_mut;
pub(crate) use impl_pystimulus_for_wrapper;
