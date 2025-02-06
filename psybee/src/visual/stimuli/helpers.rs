use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    StrokeStyle, WrappedStimulus,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::Window,
};
use psybee_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::affine::Affine;
use renderer::brushes::{Brush, Gradient};
use renderer::colors::RGBA;
use renderer::prelude::{FillStyle, Style};
use renderer::shapes::Geom;
use uuid::Uuid;

use super::LinRgba;
use crate::visual::geometry::Shape;
use renderer::vello_backend::VelloFont;
use renderer::VelloScene;

pub(crate) fn create_fill_brush(
    fill_color: &Option<LinRgba>,
    stroke_style: &Option<StrokeStyle>,
    stroke_color: &Option<LinRgba>,
    stroke_width: &Option<Size>,
    gradient: &Option<Gradient>,
    // image: Option<Image>,
) -> Brush {
    // gradient takes precedence over fill_color
    if let Some(gradient) = gradient {
        Brush::Gradient(gradient.clone())
    } else if let Some(fill_color) = fill_color {
        Brush::Solid(fill_color.clone().into())
    } else {
        Brush::Solid(RGBA::new(0.0, 0.0, 0.0, 0.0))
    }
}
// if let Some(image) = image {
