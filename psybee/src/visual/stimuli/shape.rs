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
use renderer::brushes::Brush;
use renderer::colors::RGBA;
use renderer::prelude::{FillStyle, Style};
use renderer::shapes::Geom;
use uuid::Uuid;

use crate::prelude::color::IntoLinRgba;
use crate::visual::color::LinRgba;
use crate::visual::geometry::Shape;
use renderer::vello_backend::VelloFont;
use renderer::VelloScene;

#[derive(StimulusParams, Clone, Debug)]
pub struct ShapeParams {
    pub shape: Shape,
    pub x: Size,
    pub y: Size,
    pub fill_color: Option<LinRgba>,
    pub stroke_style: Option<StrokeStyle>,
    pub stroke_color: Option<LinRgba>,
    pub stroke_width: Option<Size>,
    pub alpha: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ShapeStimulus {
    id: uuid::Uuid,
    params: ShapeParams,

    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl ShapeStimulus {
    pub fn new(
        shape: Shape,
        x: Size,
        y: Size,
        fill_color: Option<LinRgba>,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<LinRgba>,
        stroke_width: Option<Size>,
        alpha: Option<f64>,

        transform: Transformation2D,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            params: ShapeParams {
                shape,
                x,
                y,
                fill_color,
                stroke_style,
                stroke_color,
                stroke_width,
                alpha,
            },
            transform,
            animations: Vec::new(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "ShapeStimulus", extends=PyStimulus)]
pub struct PyShapeStimulus();

#[pymethods]
impl PyShapeStimulus {
    #[new]
    #[pyo3(signature = (
        shape,
        x = IntoSize(Size::Pixels(0.0)),
        y = IntoSize(Size::Pixels(0.0)),
        fill_color = None,
        stroke_style = None,
        stroke_color = None,
        stroke_width = None,
        alpha = None,
        transform = Transformation2D::Identity()
    ))]
    fn __new__(
        shape: Shape,
        x: IntoSize,
        y: IntoSize,
        fill_color: Option<IntoLinRgba>,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<IntoLinRgba>,
        stroke_width: Option<IntoSize>,
        alpha: Option<f64>,
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(ShapeStimulus::new(
                shape,
                x.into(),
                y.into(),
                fill_color.map(|f| f.into()),
                stroke_style,
                stroke_color.map(|s| s.into()),
                stroke_width.map(|s| s.into()),
                alpha,
                transform,
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyShapeStimulus, ShapeStimulus);

impl Stimulus for ShapeStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn draw(&mut self, scene: &mut VelloScene, window: &Window) {
        if !self.visible {
            return;
        }

        let x = self.params.x.eval(&window.physical_properties) as f64;
        let y = self.params.y.eval(&window.physical_properties) as f64;

        let fill_brush = super::helpers::create_fill_brush(
            &self.params.fill_color,
            &self.params.stroke_style,
            &self.params.stroke_color,
            &self.params.stroke_width,
            &None,
        );

        let stroke_color = self
            .params
            .stroke_color
            .clone()
            .unwrap_or(LinRgba::new(0.0, 0.0, 0.0, 0.0));

        let stroke_brush = renderer::brushes::Brush::Solid(stroke_color.into());

        let stroke_width = self.params.stroke_width.clone().unwrap_or(Size::Pixels(0.0));
        let stroke_width = stroke_width.eval(&window.physical_properties) as f64;

        let stroke_options = renderer::styles::StrokeOptions::new(stroke_width);

        match &self.params.shape {
            Shape::Circle { x, y, radius } => {
                let x = x.eval(&window.physical_properties) as f64;
                let y = y.eval(&window.physical_properties) as f64;
                let radius = radius.eval(&window.physical_properties) as f64;

                let shape = renderer::shapes::Circle {
                    center: renderer::shapes::Point { x, y },
                    radius: radius,
                };

                scene.draw(Geom {
                    style: Style::Fill(FillStyle::NonZero),
                    shape: shape.clone(),
                    brush: fill_brush,
                    transform: Affine::identity(),
                    brush_transform: None,
                });

                scene.draw(Geom {
                    style: Style::Stroke(stroke_options),
                    shape: shape,
                    brush: stroke_brush,
                    transform: Affine::identity(),
                    brush_transform: None,
                });
            }
            Shape::Rectangle { x, y, width, height } => {
                let x = x.eval(&window.physical_properties) as f64;
                let y = y.eval(&window.physical_properties) as f64;
                let width = width.eval(&window.physical_properties) as f64;
                let height = height.eval(&window.physical_properties) as f64;

                let shape = renderer::shapes::Rectangle {
                    a: renderer::shapes::Point { x, y },
                    b: renderer::shapes::Point {
                        x: x + width,
                        y: y + height,
                    },
                };

                scene.draw(Geom {
                    style: Style::Fill(FillStyle::NonZero),
                    shape: shape.clone(),
                    brush: fill_brush,
                    transform: Affine::identity(),
                    brush_transform: None,
                });

                scene.draw(Geom {
                    style: Style::Stroke(stroke_options),
                    shape: shape,
                    brush: stroke_brush,
                    transform: Affine::identity(),
                    brush_transform: None,
                });
            }
            Shape::Ellipse {
                x,
                y,
                radius_x,
                radius_y,
            } => {
                todo!("Render ellipse")
            }
            Shape::Line { x1, y1, x2, y2 } => {
                todo!("Render line")
            }
            Shape::Polygon { points } => {
                todo!("Render polygon")
            }
        };
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transform = transformation;
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transform.clone()
    }

    fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParamValue) {
        self.params.set_param(name, value)
    }
}
