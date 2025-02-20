use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
};
use crate::visual::geometry::Size;
use crate::visual::geometry::Transformation2D;
use crate::visual::window::PsybeeWindow;
use psybee_proc::StimulusParams;
use uuid::Uuid;

use crate::visual::color::IntoLinRgba;
use crate::visual::color::LinRgba;
use crate::visual::window::Frame;
use renderer::affine::Affine;
use renderer::brushes::Brush;
use renderer::colors::RGBA;

#[derive(StimulusParams, Clone, Debug)]
pub struct TextParams {
    pub x: Size,
    pub y: Size,
    pub text: String,
    pub font_size: Size,
    pub fill: LinRgba,
}

#[derive(Clone, Debug)]
pub struct TextStimulus {
    id: uuid::Uuid,
    params: TextParams,
    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl TextStimulus {
    pub fn new(x: Size, y: Size, text: String, font_size: Size, fill: LinRgba, transform: Transformation2D) -> Self {
        Self {
            id: Uuid::new_v4(),
            params: TextParams {
                x,
                y,
                text,
                font_size,
                fill,
            },
            transform,
            animations: Vec::new(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "TextStimulus", extends=PyStimulus)]
pub struct PyTextStimulus();

#[pymethods]
impl PyTextStimulus {
    #[new]
    #[pyo3(signature = (
        x,
        y,
        text,
        font_size,
        fill = IntoLinRgba::new(0.0, 0.0, 0.0, 1.0),
        transform = Transformation2D::Identity()
    ))]
    fn __new__(
        x: IntoSize,
        y: IntoSize,
        text: String,
        font_size: IntoSize,
        fill: IntoLinRgba,
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus::new(TextStimulus::new(
                x.into(),
                y.into(),
                text,
                font_size.into(),
                fill.into(),
                transform,
            )),
        )
    }
}

impl_pystimulus_for_wrapper!(PyTextStimulus, TextStimulus);

impl Stimulus for TextStimulus {
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

        // convert physical units to pixels
        let pos_x = self.params.x.eval(window_size, screen_props) as f64;
        let pos_y = self.params.y.eval(window_size, screen_props) as f64;
        let font_size = self.params.font_size.eval(window_size, screen_props) as f64;

        let trans_mat = self.transform.eval(window_size, screen_props);

        let fill_color: RGBA = self.params.fill.into();

        let formated_text = frame.scene_mut().draw_formated_text(
            &self.params.text,
            pos_x,
            pos_y,
            font_size,
            fill_color,
            trans_mat.into(),
        );
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
