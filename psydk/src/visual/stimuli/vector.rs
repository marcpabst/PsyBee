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

#[derive(StimulusParams, Clone, Debug)]
pub struct VectorParams {
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub opacity: f64,
}

#[derive(Clone, Debug)]
pub struct VectorStimulus {
    id: uuid::Uuid,

    params: VectorParams,

    vector: renderer::prerenderd_scene::PrerenderedScene,
    transformation: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl VectorStimulus {
    pub fn from_svg_str(svg_str: &str, params: VectorParams) -> Self {
        let vector = renderer::prerenderd_scene::PrerenderedScene::from_svg_string(svg_str, Affine::identity());

        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            animations: Vec::new(),
            visible: true,
            vector,
            params,
        }
    }

    pub fn from_svg_path(svg_path: &str, params: VectorParams) -> Self {
        let svg_str = std::fs::read_to_string(svg_path).unwrap();
        Self::from_svg_str(&svg_str, params)
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "VectorStimulus", extends=PyStimulus)]
pub struct PyVectorStimulus();

#[pymethods]
impl PyVectorStimulus {
    #[new]
    #[pyo3(signature = (
        svg_path,
        x = IntoSize(Size::Pixels(0.0)),
        y = IntoSize(Size::Pixels(0.0)),
        width = IntoSize(Size::Pixels(100.0)),
        height = IntoSize(Size::Pixels(100.0)),
        opacity = 1.0
    ))]
    fn __new__(
        svg_path: &str,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        opacity: f64,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(VectorStimulus::from_svg_path(
                svg_path,
                VectorParams {
                    x: x.into(),
                    y: y.into(),
                    width: width.into(),
                    height: height.into(),
                    opacity,
                },
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyVectorStimulus, VectorStimulus);

impl Stimulus for VectorStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&mut self, scene: &mut VelloScene, window: &psydkWindow) {
        if !self.visible {
            return;
        }

        // convert physical units to pixels
        let x = self.params.x.eval(&window.physical_properties) as f64;
        let y = self.params.y.eval(&window.physical_properties) as f64;

        let width = self.params.width.eval(&window.physical_properties) as f64;
        let height = self.params.height.eval(&window.physical_properties) as f64;

        // create a transformation matrix that scales the vector to the correct size
        let vector_width = self.vector.width;
        let vector_height = self.vector.height;

        let transformation_mat: Affine = self.transformation.eval(&window.physical_properties).into();
        let translation_mat = Affine::translate(x, y);
        let scale_mat = Affine::scale_xy(width / vector_width, height / vector_height);

        // set the transformation matrix
        &self
            .vector
            .set_transform((scale_mat * translation_mat * transformation_mat).into());

        scene.draw(&self.vector);
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

    fn contains(&self, x: Size, y: Size, window: &WrappedWindow) -> bool {
        let window = window.inner();
        let ix = self.params.x.eval(&window.physical_properties);
        let iy = self.params.y.eval(&window.physical_properties);
        let width = self.params.width.eval(&window.physical_properties);
        let height = self.params.height.eval(&window.physical_properties);

        let trans_mat = self.transformation.eval(&window.physical_properties);

        let x = x.eval(&window.physical_properties);
        let y = y.eval(&window.physical_properties);

        // apply transformation by multiplying the point with the transformation matrix
        let p = nalgebra::Vector3::new(x, y, 1.0);
        let p_new = trans_mat * p;

        // check if the point is inside the rectangle
        p_new[0] >= ix && p_new[0] <= ix + width && p_new[1] >= iy && p_new[1] <= iy + height
    }

    fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParamValue) {
        self.params.set_param(name, value)
    }
}
