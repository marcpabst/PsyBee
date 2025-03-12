use std::sync::Arc;

use psydk_proc::{FromPyStr, StimulusParams};
use renderer::DynamicBitmap;
use renderer::{affine::Affine, brushes::Brush, colors::RGBA};
use strum::EnumString;
use uuid::Uuid;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    StrokeStyle,
};
use crate::visual::{
    color::{IntoLinRgba, LinRgba},
    geometry::{Shape, Size, Transformation2D},
    window::Frame,
};

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
/// A stimulus that displays a shape.
///
/// Parameters
/// ----------
/// shape : Shape
///     The shape to display.
/// x : Size, optional
///     The x-coordinate of the center of the shape.
/// y : Size, optional
///     The y-coordinate of the center of the shape.
/// fill_color : Union[LinRgba, (float, float, float), (float, float, float, float), str], optional
///    The fill color of the shape.
/// stroke_style : StrokeStyle, optional
///    The stroke style of the shape.
/// stroke_color : Union[LinRgba, (float, float, float), (float, float, float, float), str], optional
///   The stroke color of the shape.
/// stroke_width : Union[Size, float], optional
///  The stroke width of the shape.
/// alpha : float, optional
///  The alpha channel of the shape.
/// transform : Transformation2D, optional
/// The transformation of the shape.
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
    /// A stimulus that displays a shape.
    ///
    /// Parameters
    /// ----------
    /// shape : Shape
    ///     The shape to display.
    /// x : Size, optional
    ///     The x-coordinate of the center of the shape.
    /// y : Size, optional
    ///     The y-coordinate of the center of the shape.
    /// fill_color : Union[LinRgba, (float, float, float), (float, float, float, float), str], optional
    ///    The fill color of the shape.
    /// stroke_style : StrokeStyle, optional
    ///    The stroke style of the shape.
    /// stroke_color : Union[LinRgba, (float, float, float), (float, float, float, float), str], optional
    ///   The stroke color of the shape.
    /// stroke_width : Union[Size, float], optional
    ///    The stroke width of the shape.
    /// alpha : float, optional
    ///    The alpha channel of the shape.
    /// transform : Transformation2D, optional
    ///    The transformation of the shape.
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
            PyStimulus::new(ShapeStimulus::new(
                shape,
                x.into(),
                y.into(),
                fill_color.map(|f| f.into()),
                stroke_style,
                stroke_color.map(|s| s.into()),
                stroke_width.map(|s| s.into()),
                alpha,
                transform,
            )),
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

    fn draw(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let window = frame.window();
        let window_state = window.lock_state();
        let windows_size = window_state.size;
        let screen_props = window_state.physical_screen;

        let x_origin = self.params.x.eval(windows_size, screen_props) as f64;
        let y_origin = self.params.y.eval(windows_size, screen_props) as f64;

        let fill_brush = super::helpers::create_fill_brush(
            &self.params.fill_color,
            &self.params.stroke_style,
            &self.params.stroke_color,
            &self.params.stroke_width,
            &None,
        );

        let stroke_color = self.params.stroke_color.unwrap_or(LinRgba::new(0.0, 0.0, 0.0, 0.0));

        let stroke_brush = renderer::brushes::Brush::Solid(stroke_color.into());

        let stroke_width = self.params.stroke_width.clone().unwrap_or(Size::Pixels(0.0));
        let stroke_width = stroke_width.eval(windows_size, screen_props) as f64;

        let stroke_options = renderer::styles::StrokeStyle::new(stroke_width);

        match &self.params.shape {
            Shape::Circle { x, y, radius } => {
                let x = x.eval(windows_size, screen_props) as f64;
                let y = y.eval(windows_size, screen_props) as f64;
                let radius = radius.eval(windows_size, screen_props) as f64;

                // move by x_origin and y_origin
                let x = x + x_origin;
                let y = y + y_origin;

                let shape = renderer::shapes::Shape::circle((x, y), radius);

                frame.scene_mut().draw_shape_fill(shape, fill_brush.clone(), None, None);

                frame
                    .scene_mut()
                    .draw_shape_stroke(shape, stroke_brush, stroke_options, None, None);
            }
            Shape::Rectangle { x, y, width, height } => {
                let x = x.eval(windows_size, screen_props) as f64;
                let y = y.eval(windows_size, screen_props) as f64;
                let width = width.eval(windows_size, screen_props) as f64;
                let height = height.eval(windows_size, screen_props) as f64;

                // move by x_origin and y_origin
                let x = x + x_origin;
                let y = y + y_origin;

                let shape = renderer::shapes::Shape::rectangle((x, y), width, height);

                frame.scene_mut().draw_shape_fill(shape, fill_brush.clone(), None, None);

                frame
                    .scene_mut()
                    .draw_shape_stroke(shape, stroke_brush, stroke_options, None, None);
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
                let x1 = x1.eval(windows_size, screen_props) as f64;
                let y1 = y1.eval(windows_size, screen_props) as f64;
                let x2 = x2.eval(windows_size, screen_props) as f64;
                let y2 = y2.eval(windows_size, screen_props) as f64;

                // move by x_origin and y_origin
                let x1 = x1 + x_origin;
                let y1 = y1 + y_origin;
                let x2 = x2 + x_origin;
                let y2 = y2 + y_origin;

                let shape = renderer::shapes::Shape::line((x1, y1), (x2, y2));

                frame
                    .scene_mut()
                    .draw_shape_stroke(shape, stroke_brush, stroke_options, None, None);
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
