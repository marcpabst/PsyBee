use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::Window,
};
use psybee_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::prelude::*;
use uuid::Uuid;

use renderer::vello_backend::VelloFont;

use super::Rgba;

#[derive(StimulusParams, Clone, Debug)]
pub struct TextParams {
    pub x: Size,
    pub y: Size,
    pub text: String,
    pub font_size: Size,
    pub fill: Rgba,
}

#[derive(Clone, Debug)]
pub struct TextStimulus {
    id: uuid::Uuid,
    params: TextParams,
    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
    font: VelloFont,
}

impl TextStimulus {
    pub fn new(x: Size, y: Size, text: String, font_size: Size, fill: Rgba, transform: Transformation2D) -> Self {
        // load font
        let font_data =
            include_bytes!("../../../../bubblesdemo/src/bubblesdemo/resources/TradeWinds-Regular.ttf");
        let font = VelloFont::from_bytes(font_data);

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
            font,
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
        fill = Rgba::new(0.0, 0.0, 0.0, 1.0),
        transform = Transformation2D::Identity()
    ))]
    fn __new__(
        x: IntoSize,
        y: IntoSize,
        text: String,
        font_size: IntoSize,
        fill: Rgba,
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(TextStimulus::new(
                x.into(),
                y.into(),
                text,
                font_size.into(),
                fill,
                transform,
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyTextStimulus, TextStimulus);

impl Stimulus for TextStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn draw(&self, scene: &mut VelloScene, window: &Window) {
        if !self.visible {
            return;
        }

        // convert physical units to pixels
        let pos_x = self.params.x.eval(&window.physical_properties) as f64;
        let pos_y = self.params.y.eval(&window.physical_properties) as f64;
        let font_size = self.params.font_size.eval(&window.physical_properties) as f32;

        let text = FormatedText {
            x: pos_x,
            y: pos_y / 3.0,
            text: self.params.text.clone(),
            size: font_size,
            color: self.params.fill.into(),
            weight: 100.0,
            font: self.font.clone(),
            style: FontStyle::Normal,
            alignment: Alignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            transform: self.transform.eval(&window.physical_properties).into(),
            glyph_transform: None,
        };

        scene.draw(text);
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
