use std::sync::Arc;

use psybee_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::{
    affine::Affine,
    brushes::{Brush, Gradient},
    colors::RGBA,
};
use uuid::Uuid;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, LinRgba, PyStimulus, Stimulus, StimulusParamValue,
    StimulusParams, StrokeStyle,
};
use crate::visual::{geometry::Size, window::Window};

pub(crate) fn create_fill_brush<'a>(
    fill_color: &Option<LinRgba>,
    stroke_style: &Option<StrokeStyle>,
    stroke_color: &Option<LinRgba>,
    stroke_width: &Option<Size>,
    gradient: &Option<Gradient>,
    // image: Option<Image>,
) -> Brush<'a> {
    // gradient takes precedence over fill_color
    if let Some(gradient) = gradient {
        Brush::Gradient(gradient.clone())
    } else if let Some(fill_color) = fill_color {
        Brush::Solid((*fill_color).into())
    } else {
        Brush::Solid(RGBA::new(0.0, 0.0, 0.0, 0.0))
    }
}
// if let Some(image) = image {
