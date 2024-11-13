use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    WrappedStimulus,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::Window,
};

use psybee_proc::StimulusParams;

use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::prelude::*;
use uuid::Uuid;

#[derive(StimulusParams, Clone, Debug)]
pub struct ImageParams {
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub opacity: f64,
    pub image_x: Size,
    pub image_y: Size,
}

#[derive(Clone, Debug)]
pub struct ImageStimulus {
    id: uuid::Uuid,

    params: ImageParams,

    image: super::WrappedImage,
    image_fit_mode: ImageFitMode,
    transformation: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl ImageStimulus {
    pub fn from_image(image: super::WrappedImage, params: ImageParams) -> Self {
        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            animations: Vec::new(),
            visible: true,
            image: image,
            image_fit_mode: ImageFitMode::Fill,
            params,
        }
    }

    pub fn from_path(src: String, params: ImageParams) -> Self {
        let image = super::WrappedImage::from_path(src).unwrap();
        Self::from_image(image, params)
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "ImageStimulus", extends=PyStimulus)]
pub struct PyImageStimulus();

#[pymethods]
impl PyImageStimulus {
    #[new]
    #[pyo3(signature = (
        image,
        x,
        y,
        width,
        height,
        opacity = 1.0
    ))]
    fn __new__(
        image: &super::WrappedImage,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        opacity: f64,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(ImageStimulus::from_image(
                image.clone(),
                ImageParams {
                    x: x.into(),
                    y: y.into(),
                    width: width.into(),
                    height: height.into(),
                    image_x: 0.0.into(),
                    image_y: 0.0.into(),
                    opacity,
                },
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyImageStimulus, ImageStimulus);

impl Stimulus for ImageStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&self, scene: &mut VelloScene, window: &Window) {
        if !self.visible {
            return;
        }

        // convert physical units to pixels
        let x = self.params.x.eval(&window.physical_properties) as f64;
        let y = self.params.y.eval(&window.physical_properties) as f64;
        let width = self.params.width.eval(&window.physical_properties) as f64;
        let height = self.params.height.eval(&window.physical_properties) as f64;

        let image_offset_x = self.params.image_x.eval(&window.physical_properties) as f64;
        let image_offset_y = self.params.image_y.eval(&window.physical_properties) as f64;

        let trans_mat = self.transformation.eval(&window.physical_properties);

        scene.draw(Geom::new_image(
            self.image.inner().clone(),
            x,
            y,
            width,
            height,
            trans_mat.into(),
            image_offset_x,
            image_offset_y,
            ImageFitMode::Fill,
            Extend::Pad,
        ));
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
