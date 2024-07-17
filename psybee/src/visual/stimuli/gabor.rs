use super::PyStimulus;
use pyo3::prelude::*;
use uuid::Uuid;

use super::Stimulus;
use crate::{
    renderer::{Colour, Geom, Material, Point2D, Primitive, Renderable, TessellationOptions},
    visual::{geometry::Size, window::Window},
};

#[derive(Clone, Debug)]
pub struct GaborStimulus {
    id: uuid::Uuid,

    origin: (Size, Size),
    transformation: crate::visual::geometry::Transformation2D,
    visible: bool,

    pub size: Size,
    pub frequency: f32,
    pub phase: f32,
    pub sigma: f32,
    pub orientation: f32,
}

impl Stimulus for GaborStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&self, window: &Window) -> Vec<Renderable> {
        // create the material (uniform colour for now)
        let material = Material::Colour(Colour::WHITE);

        let trans_mat = self
            .transformation
            .to_transformation_matrix(&window.physical_properties);

        // create the drawables
        let patch_geom = Geom::new(
            Primitive::Circle {
                center: Point2D::new(0.0, 0.0),
                radius: self.size.eval(&window.physical_properties),
            },
            material,
            Some(trans_mat.into()),
            vec![],
            TessellationOptions::Fill,
        );

        vec![Renderable::Geom(patch_geom)]
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_origin(&mut self, x: Size, y: Size) {
        self.origin = (x, y);
    }

    fn origin(&self) -> (Size, Size) {
        self.origin.clone()
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }
}

impl GaborStimulus {
    pub fn new(size: Size, frequency: f32, phase: f32, sigma: f32, orientation: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            origin: (Size::Pixels(0.0), Size::Pixels(0.0)),
            visible: true,
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            size,
            frequency,
            phase,
            sigma,
            orientation,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "GaborStimulus", frozen, extends=PyStimulus)]
pub struct PyGaborStimulus();

#[pymethods]
impl PyGaborStimulus {
    #[new]
    fn __new__(size: Size, frequency: f32, phase: f32, sigma: f32, orientation: f32) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Box::new(GaborStimulus::new(size, frequency, phase, sigma, orientation))),
        )
    }
}
