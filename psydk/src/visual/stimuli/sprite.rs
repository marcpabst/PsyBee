use std::sync::Arc;

use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
    WrappedImage, WrappedStimulus,
};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::psydkWindow,
};

use psydk_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::{image::GenericImageView, prelude::*};
use uuid::Uuid;

#[derive(StimulusParams, Clone, Debug)]
pub struct SpriteParams {
    pub x: Size,
    pub y: Size,
    pub fps: f64,
    pub repeat: bool,
    pub width: Size,
    pub height: Size,
}

#[derive(Clone, Debug)]
pub struct SpriteStimulus {
    id: uuid::Uuid,

    params: SpriteParams,

    images: Vec<WrappedImage>,
    start_time: std::time::Instant,

    transformation: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl SpriteStimulus {
    pub fn new(images: Vec<WrappedImage>, params: SpriteParams) -> Self {
        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            animations: Vec::new(),
            visible: true,
            start_time: std::time::Instant::now(),
            images,
            params,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "SpriteStimulus", extends=PyStimulus)]
pub struct PySpriteStimulus();

#[pymethods]
impl PySpriteStimulus {
    #[new]
    #[pyo3(signature = (
        images,
        fps,
        x,
        y,
        width,
        height,
        repeat = true
    ))]

    fn __new__(
        images: Vec<WrappedImage>,
        fps: f64,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        repeat: bool,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Arc::new(std::sync::Mutex::new(SpriteStimulus::new(
                images,
                SpriteParams {
                    x: x.into(),
                    y: y.into(),
                    fps,
                    repeat,
                    width: width.into(),
                    height: height.into(),
                },
            )))),
        )
    }
}

impl_pystimulus_for_wrapper!(PySpriteStimulus, SpriteStimulus);

impl Stimulus for SpriteStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&mut self, scene: &mut VelloScene, window: &psydkWindow) {
        if !self.visible {
            return;
        }

        // work out the current frame
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let n_frame = (elapsed * self.params.fps as f32) as usize;

        let n_frame = if self.params.repeat {
            n_frame % self.images.len()
        } else {
            n_frame.min(self.images.len() - 1)
        };

        let image = &self.images[n_frame];

        // convert physical units to pixels
        let x = self.params.x.eval(&window.physical_properties) as f64;
        let y = self.params.y.eval(&window.physical_properties) as f64;
        let width = self.params.width.eval(&window.physical_properties) as f64;
        let height = self.params.height.eval(&window.physical_properties) as f64;

        let trans_mat = self.transformation.eval(&window.physical_properties);

        scene.draw(Geom::new_image(
            image.inner().clone(),
            x,
            y,
            width,
            height,
            trans_mat.into(),
            0.0,
            0.0,
            ImageFitMode::Fill,
            Extend::Repeat,
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
