use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    WrappedStimulus,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::psydkWindow,
};
use psydk_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::prelude::*;
use uuid::Uuid;

use renderer::vello_backend::VelloFont;

use super::LinRgba;

#[derive(StimulusParams, Clone, Debug)]
pub struct GridParams {
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
}

#[derive(Clone, Debug)]
pub struct GridStimulus {
    id: uuid::Uuid,
    params: GridParams,
    elements: Vec<WrappedStimulus>,
    cols: Option<usize>,
    rows: Option<usize>,
    widths: f64,
    heights: f64,
    anchor: (f64, f64),
    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl GridStimulus {
    pub fn new(
        elements: Vec<WrappedStimulus>,
        x: Size,
        y: Size,
        width: Size,
        height: Size,
        cols: Option<usize>,
        rows: Option<usize>,
        anchor: (f64, f64),
        transform: Transformation2D,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            params: GridParams { x, y, width, height },
            transform,
            animations: Vec::new(),
            visible: true,
            elements,
            cols,
            rows,
            widths: 0.0,
            heights: 0.0,
            anchor,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "GridStimulus", extends=PyStimulus)]
pub struct PyGridStimulus();

#[pymethods]
impl PyGridStimulus {
    #[new]
    #[pyo3(signature = (
        elements,
        x = IntoSize(Size::Pixels(0.0)),
        y = IntoSize(Size::Pixels(0.0)),
        width = IntoSize(Size::ScreenWidth(1.0)),
        height = IntoSize(Size::ScreenHeight(1.0)),
        cols = None,
        rows = None,
        anchor = (0.5, 0.5),
        transform = Transformation2D::Identity()
    ))]
    fn __new__(
        elements: Vec<PyStimulus>,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        cols: Option<usize>,
        rows: Option<usize>,
        anchor: (f64, f64),
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        let elements = elements.into_iter().map(|e| e.0).collect();
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(GridStimulus::new(
                elements,
                x.into(),
                y.into(),
                width.into(),
                height.into(),
                cols,
                rows,
                anchor,
                transform,
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyGridStimulus, GridStimulus);

impl Stimulus for GridStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn draw(&mut self, scene: &mut VelloScene, window: &psydkWindow) {
        // work out number of columns and rows if not specified
        // if only one is specified, calculate the other based on the number of elements
        // if neither are specified, calculate the number of columns based on the number of elements,
        // so that the grid is as square as possible
        let cols = self.cols.unwrap_or_else(|| {
            self.rows
                .map(|r| (self.elements.len() as f64 / r as f64).ceil() as usize)
                .unwrap_or_else(|| (self.elements.len() as f64).sqrt().ceil() as usize)
        });
        let rows = self
            .rows
            .unwrap_or_else(|| (self.elements.len() as f64 / cols as f64).ceil() as usize);

        // calculate the width and height of each cell
        let cell_width = self.params.width.eval(&window.physical_properties) / cols as f32;
        let cell_height = self.params.height.eval(&window.physical_properties) / rows as f32;

        // draw each element in the grid
        for (i, element) in self.elements.iter_mut().enumerate() {
            let col = i % cols;
            let row = i / cols;

            let x = self.params.x.eval(&window.physical_properties) + col as f32 * cell_width;
            let y = self.params.y.eval(&window.physical_properties) + row as f32 * cell_height;

            let mut stimulus = element.lock().unwrap();
            stimulus.draw(scene, window);
        }
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
