use std::sync::Arc;

use psybee_proc::{FromPyStr, StimulusParams};
use renderer::{
    affine::Affine, brushes::Brush, colors::RGBA, prelude::ImageSampling, renderer::RendererFactory,
    styles::ImageFitMode, DynamicBitmap,
};
use strum::EnumString;
use uuid::Uuid;

use renderer::prelude::*;

unsafe impl Send for PatternStimulus {}

use super::{
    animations::Animation, helpers, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue,
    StimulusParams, StrokeStyle,
};
use crate::visual::{
    color::{IntoLinRgba, LinRgba},
    geometry::{Shape, Size, Transformation2D},
    window::Frame,
};

#[derive(EnumString, Debug, Clone, Copy, PartialEq, FromPyStr)]
#[strum(serialize_all = "snake_case")]
pub enum FillPattern {
    Uniform,
    Square,
    Sinosoidal,
    Checkerboard,
}

#[derive(StimulusParams, Clone, Debug)]
pub struct PatternParams {
    pub shape: Shape,
    pub x: Size,
    pub y: Size,
    pub phase_x: f64,
    pub phase_y: f64,
    pub cycle_length: Size,
    pub fill_color: Option<LinRgba>,
    pub background_color: Option<LinRgba>,
    pub stroke_style: Option<StrokeStyle>,
    pub stroke_color: Option<LinRgba>,
    pub stroke_width: Option<Size>,
    pub alpha: Option<f64>,
}

#[derive(Debug)]
pub struct PatternStimulus {
    id: uuid::Uuid,
    params: PatternParams,
    fill_pattern: FillPattern,

    gradient_colors: Option<Vec<LinRgba>>,
    pattern_image: Option<DynamicBitmap>,

    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl PatternStimulus {
    pub fn new(
        shape: Shape,
        x: Size,
        y: Size,
        phase_x: f64,
        phase_y: f64,
        cycle_length: Size,
        fill_color: Option<LinRgba>,
        background_color: Option<LinRgba>,
        pattern: Option<FillPattern>,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<LinRgba>,
        stroke_width: Option<Size>,
        alpha: Option<f64>,

        transform: Transformation2D,
        renderer_factory: &dyn RendererFactory,
    ) -> Self {
        // for now, create a 2x2 checkerboard pattern
        let image_2x2_data = vec![0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0];
        let image_2x2 = renderer::image::RgbImage::from_raw(2, 2, image_2x2_data).unwrap();

        let pattern_image = renderer_factory.create_bitmap(renderer::image::DynamicImage::ImageRgb8(image_2x2));

        Self {
            id: Uuid::new_v4(),
            params: PatternParams {
                shape,
                x,
                y,
                phase_x,
                phase_y,
                cycle_length,
                fill_color,
                background_color,
                stroke_style,
                stroke_color,
                stroke_width,
                alpha,
            },
            fill_pattern: pattern.unwrap_or(FillPattern::Uniform),
            gradient_colors: None,
            pattern_image: Some(pattern_image),

            transform,
            animations: Vec::new(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "PatternStimulus", extends=PyStimulus)]
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
pub struct PyPatternStimulus();

#[pymethods]
impl PyPatternStimulus {
    #[new]
    #[pyo3(signature = (
        shape,
        x = IntoSize(Size::Pixels(0.0)),
        y = IntoSize(Size::Pixels(0.0)),
        phase_x = 0.0,
        phase_y = 0.0,
        cycle_length = IntoSize(Size::Pixels(100.0)),
        fill_color = None,
        background_color = None,
        pattern = None,
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
        py: Python,
        shape: Shape,
        x: IntoSize,
        y: IntoSize,
        phase_x: f64,
        phase_y: f64,
        cycle_length: IntoSize,
        fill_color: Option<IntoLinRgba>,
        background_color: Option<IntoLinRgba>,
        pattern: Option<FillPattern>,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<IntoLinRgba>,
        stroke_width: Option<IntoSize>,
        alpha: Option<f64>,
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        let renderer_factory = helpers::get_renderer_factory(py).unwrap();
        (
            Self(),
            PyStimulus::new(PatternStimulus::new(
                shape,
                x.into(),
                y.into(),
                phase_x,
                phase_y,
                cycle_length.into(),
                fill_color.map(|s| s.into()),
                background_color.map(|s| s.into()),
                pattern,
                stroke_style,
                stroke_color.map(|s| s.into()),
                stroke_width.map(|s| s.into()),
                alpha,
                transform,
                renderer_factory.inner(),
            )),
        )
    }
}

impl_pystimulus_for_wrapper!(PyPatternStimulus, PatternStimulus);

impl Stimulus for PatternStimulus {
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

        let cycle_length = self.params.cycle_length.eval(windows_size, screen_props);

        let shift_x = (self.params.phase_x % 360.0) / 360.0 * cycle_length as f64;
        let shift_y = (self.params.phase_y % 360.0) / 360.0 * cycle_length as f64;

        let fill_brush = Brush::Image {
            image: self.pattern_image.as_ref().unwrap(),
            start: (shift_x, shift_y).into(),
            fit_mode: ImageFitMode::Exact {
                width: cycle_length,
                height: cycle_length,
            },
            sampling: ImageSampling::Nearest,
            edge_mode: (Extend::Repeat, Extend::Repeat),
            transform: None,
            alpha: self.params.alpha.map(|a| a as f32),
        };

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
