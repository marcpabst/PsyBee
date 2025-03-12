use std::sync::Arc;

use psydk_proc::{FromPyStr, StimulusParams};
use pyo3::{pyclass, pymethods};
use renderer::{
    affine::Affine,
    brushes::{Brush, Extend, Gradient, GradientKind},
    colors::RGBA,
    shapes::{Point, Shape},
    styles::BlendMode,
};
use strum::EnumString;
use uuid::Uuid;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    StrokeStyle,
};
use crate::visual::{
    color::LinRgba,
    geometry::{Anchor, Size, Transformation2D},
    window::Frame,
};

#[derive(EnumString, Debug, Clone, Copy, PartialEq, FromPyStr)]
pub enum Pattern {
    Sine,
    Square,
}

#[derive(EnumString, Debug, Clone, Copy, PartialEq, FromPyStr)]
pub enum ColorInterpolation {
    Linear,
    Srgb,
}

#[derive(StimulusParams, Clone, Debug)]
pub struct GaborParams {
    pub cx: Size,
    pub cy: Size,
    pub radius: Size,
    pub cycle_length: Size,
    pub phase: f64,
    pub sigma: Size,
    pub orientation: f64,
    pub stroke_style: Option<StrokeStyle>,
    pub stroke_color: Option<LinRgba>,
    pub stroke_width: Option<Size>,
    pub alpha: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct GaborStimulus {
    id: uuid::Uuid,

    params: GaborParams,

    pattern_colors: Vec<RGBA>,
    gaussian_colors: Option<Vec<RGBA>>,
    pattern: Pattern,
    color_interpolation: ColorInterpolation,

    transformation: Transformation2D,
    anchor: Anchor,
    animations: Vec<Animation>,
    visible: bool,
}

impl GaborStimulus {
    pub fn new(
        cx: Size,
        cy: Size,
        radius: Size,
        pattern: Pattern,
        cycle_length: Size,
        phase: f64,
        sigma: Size,
        orientation: f64,
        anchor: Anchor,
        color_interpolation: ColorInterpolation,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<LinRgba>,
        stroke_width: Option<Size>,
        alpha: Option<f64>,
    ) -> Self {
        let gaussian_colors: Vec<RGBA> = (0..128)
            .map(|i| {
                let sigma: f32 = 0.25;
                // we need a Gaussian function scaled to values between 0 and 1
                // i.e., f(x) = exp(-x^2 / (2 * sigma^2))
                let x = (i as f32 / 128.0);
                let t = (-x.powi(2) / (2.0 * sigma.powi(2))).exp();

                RGBA {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: t,
                }
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            anchor,
            animations: Vec::new(),
            visible: true,

            params: GaborParams {
                cx,
                cy,
                radius,
                cycle_length,
                phase,
                sigma,
                orientation,
                stroke_style,
                stroke_color,
                stroke_width,
                alpha,
            },
            pattern_colors: Self::create_sine_colors(256),
            gaussian_colors: Some(gaussian_colors),
            pattern,
            color_interpolation,
        }
    }

    fn create_sine_colors(len: usize) -> Vec<RGBA> {
        let sine_grating_colors: Vec<RGBA> = (0..len)
            .map(|i| {
                let x = i as f32 / 256.0 * 1.0 * std::f32::consts::PI;
                let t = x.sin();
                RGBA {
                    r: t,
                    g: t,
                    b: t,
                    a: 1.0,
                }
            })
            .collect();
        sine_grating_colors
    }

    fn create_square_colors(len: usize) -> Vec<RGBA> {
        let f_len = len as f32;
        let square_grating_colors: Vec<RGBA> = (0..len)
            .map(|i| {
                let t = if (i as f32) < f_len / 2.0 { 1.0 } else { 0.0 };
                RGBA {
                    r: t,
                    g: t,
                    b: t,
                    a: 1.0,
                }
            })
            .collect();
        square_grating_colors
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "GaborStimulus", extends=PyStimulus, module = "psydk.visual.stimuli")]
/// A 1D grating multiplied with a Gaussian envelope.
///
/// Parameters
/// ----------
/// cx : str or Number
///   The x-coordinate of the center of the stimulus.
/// cy : str or Number
///   The y-coordinate of the center of the stimulus.
/// radius : str or Number
///   The radius of the stimulus.
/// cycle_length : str or Number
///   The length of a single cycle of the grating.
/// sigma : str or Number
///   The standard deviation of the Gaussian envelope.
/// pattern : Literal['sine', 'square'], optional
///   The pattern of the grating (default is 'sine').
/// phase : float, optional
///   The phase of the grating (default is 0.0).
/// orientation : float, optional
///   The orientation of the grating in degrees (default is 0.0).
/// anchor : Literal['center', 'top-left', 'top-right', 'bottom-left', 'bottom-right'], optional
///   The anchor point of the stimulus (default is 'center').
/// color_interpolation : Literal['linear', 'srgb'], optional
///   The color interpolation mode (default is 'linear').
/// stroke_style : str or StrokeStyle, optional
///   The stroke style of the stimulus.
/// stroke_color : (float,float,float),  (float,float,float, float), str or LinRgba, optional
///   The stroke color of the stimulus. Either an sRGB(A) tuple or a LinRgba color.
/// stroke_width : str or Number, optional
///   With of the stroke.
/// alpha : float, optional
///   The alpha value of the stimulus.
pub struct PyGaborStimulus();

#[pymethods]
impl PyGaborStimulus {
    #[new]
    #[pyo3(signature = (
        cx,
        cy,
        radius,
        cycle_lenght,
        sigma,
        pattern = Pattern::Sine,
        phase = 0.0,
        orientation = 0.0,
        anchor = Anchor::Center,
        color_interpolation = ColorInterpolation::Linear,
        stroke_style = None,
        stroke_color = None,
        stroke_width = None,
        alpha = None
    ))]
    /// Create a new Gabor stimulus.
    fn __new__(
        cx: IntoSize,
        cy: IntoSize,
        radius: IntoSize,
        cycle_lenght: IntoSize,
        sigma: IntoSize,
        pattern: Pattern,
        phase: f64,
        orientation: f64,
        anchor: Anchor,
        color_interpolation: ColorInterpolation,
        stroke_style: Option<StrokeStyle>,
        stroke_color: Option<LinRgba>,
        stroke_width: Option<IntoSize>,
        alpha: Option<f64>,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus::new(GaborStimulus::new(
                cx.into(),
                cy.into(),
                radius.into(),
                pattern,
                cycle_lenght.into(),
                phase,
                sigma.into(),
                orientation,
                anchor,
                color_interpolation,
                stroke_style,
                stroke_color,
                stroke_width.map(Into::into),
                alpha,
            )),
        )
    }
}

impl_pystimulus_for_wrapper!(PyGaborStimulus, GaborStimulus);

impl Stimulus for GaborStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let window = frame.window();
        let window_state = window.lock_state();
        let window_size = window_state.size;
        let screen_props = window_state.physical_screen;

        let mut scene = frame.scene_mut();

        // convert physical units to pixels
        let radius = self.params.radius.eval(window_size, screen_props) as f64;
        let sigma = self.params.sigma.eval(window_size, screen_props);
        let cycle_length = self.params.cycle_length.eval(window_size, screen_props) as f64;
        let pos_x = self.params.cx.eval(window_size, screen_props) as f64;
        let pos_y = self.params.cy.eval(window_size, screen_props) as f64;

        // apply the anchor
        let bb_width = radius * 2.0;
        let bb_height = radius * 2.0;
        let (pos_x, pos_y) = self.anchor.to_center(pos_x, pos_y, bb_width, bb_height);

        let trans_mat = self.transformation.eval(window_size, screen_props);

        // convert phase into the range [0, 1] (from [0, 2π])
        let phase = self.params.phase % (2.0 * std::f64::consts::PI);
        let transl_x = phase * cycle_length;

        // transform for the brush
        let grating_transform = Affine::rotate_at(self.params.orientation, pos_x, pos_y);

        let grating_shape = Shape::circle(Point { x: pos_x, y: pos_y }, radius);

        let grating_brush = Brush::Gradient(Gradient::new_equidistant(
            Extend::Repeat,
            GradientKind::Linear {
                start: Point {
                    x: pos_x + transl_x,
                    y: pos_y,
                },
                end: Point {
                    x: pos_x + cycle_length + transl_x,
                    y: pos_y,
                },
            },
            &self.pattern_colors,
        ));

        let gaussian_shape = Shape::circle(Point { x: pos_x, y: pos_y }, radius + 1.0);

        let gaussian_brush = Brush::Gradient(Gradient::new_equidistant(
            Extend::Pad,
            GradientKind::Radial {
                center: Point { x: pos_x, y: pos_y },
                radius: (radius as f32),
            },
            self.gaussian_colors.as_deref().unwrap(),
        ));

        let transform = self.transformation.eval(window_size, screen_props);
        let alpha = self.params.alpha.unwrap_or(1.0);
        scene.start_layer(
            BlendMode::SourceOver,
            gaussian_shape,
            Some(transform.into()),
            None,
            alpha as f32,
        );
        scene.draw_shape_fill(
            gaussian_shape,
            gaussian_brush,
            Some(transform.into()),
            Some(BlendMode::SourceOver),
        );
        scene.draw_shape_fill(
            grating_shape,
            grating_brush,
            Some(transform.into()),
            Some(BlendMode::SourceIn),
        );
        scene.end_layer();

        // if the stimulus has a stroke, draw it
        if let Some(stroke_style) = &self.params.stroke_style {
            let stroke_color = self.params.stroke_color.unwrap_or(LinRgba::new(0.0, 0.0, 0.0, 1.0));
            let stroke_brush = Brush::Solid(stroke_color.into());
            let stroke_width = self.params.stroke_width.clone().unwrap_or(Size::Pixels(0.0));
            let stroke_width = stroke_width.eval(window_size, screen_props) as f64;
            let stroke_options = renderer::styles::StrokeStyle::new(stroke_width);

            let shape = Shape::circle(Point { x: pos_x, y: pos_y }, radius);
            scene.draw_shape_stroke(shape, stroke_brush, stroke_options, Some(transform.into()), None);
        }
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transformation.clone()
    }

    fn contains(&self, x: Size, y: Size, window: &Window) -> bool {
        // let cx = self.params.cx.eval(&window.physical_properties);
        // let cy = self.params.cy.eval(&window.physical_properties);
        // let radius = self.params.radius.eval(&window.physical_properties);
        // let trans_mat = self.transformation.eval(&window.physical_properties);

        // let x = x.eval(&window.physical_properties);
        // let y = y.eval(&window.physical_properties);

        // // apply transformation by multiplying the point with the transformation matrix
        // let p = nalgebra::Vector3::new(x, y, 1.0);
        // let p_new = trans_mat * p;

        // // check if the point is inside the circle
        // let dx = p_new[0] - cx;
        // let dy = p_new[1] - cy;
        // let distance = (dx * dx + dy * dy).sqrt();

        // distance <= radius
        false
    }

    fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParamValue) {
        self.params.set_param(name, value)
    }
}
