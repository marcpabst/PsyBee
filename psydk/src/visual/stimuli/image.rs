use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use psydk_proc::StimulusParams;
use pyo3::ffi::c_str;
use renderer::{
    brushes::{Brush, Extend, ImageSampling},
    shapes::Shape,
    styles::ImageFitMode,
    DynamicBitmap,
};
use uuid::Uuid;

use super::{
    animations::Animation, helpers, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue,
    StimulusParams,
};
use crate::{
    experiment::PyRendererFactory,
    visual::{
        geometry::{Anchor, Size, Transformation2D},
        window::Frame,
    },
};

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

#[derive(Debug)]
pub struct ImageStimulus {
    id: uuid::Uuid,

    params: ImageParams,

    image: DynamicBitmap,
    anchor: Anchor,
    transformation: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

unsafe impl Send for ImageStimulus {}

impl ImageStimulus {
    pub fn from_image(
        image: DynamicBitmap,
        params: ImageParams,
        transform: Option<Transformation2D>,
        anchor: Anchor,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transformation: transform.unwrap_or_else(|| Transformation2D::Identity()),
            animations: Vec::new(),
            visible: true,
            image,
            anchor,
            params,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "ImageStimulus", extends=PyStimulus)]
pub struct PyImageStimulus();

#[pymethods]
impl PyImageStimulus {
    #[new]
    #[pyo3(signature = (
        src,
        x,
        y,
        width,
        height,
        opacity = 1.0,
        anchor = Anchor::Center,
        transform = None,
        srgb = true
    ))]
    fn __new__(
        py: Python,
        src: String,
        x: IntoSize,
        y: IntoSize,
        width: IntoSize,
        height: IntoSize,
        opacity: f64,
        anchor: Anchor,
        transform: Option<Transformation2D>,
        srgb: bool,
    ) -> (Self, PyStimulus) {
        let renderer_factory = helpers::get_renderer_factory(py).unwrap();

        let bitmap = renderer_factory.create_bitmap_from_path(&src);

        (
            Self(),
            PyStimulus::new(ImageStimulus::from_image(
                bitmap,
                ImageParams {
                    x: x.into(),
                    y: y.into(),
                    width: width.into(),
                    height: height.into(),
                    image_x: 0.0.into(),
                    image_y: 0.0.into(),
                    opacity,
                },
                transform,
                anchor,
            )),
        )
    }
}

impl_pystimulus_for_wrapper!(PyImageStimulus, ImageStimulus);

impl Stimulus for ImageStimulus {
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
        let x = self.params.x.eval(window_size, screen_props);
        let y = self.params.y.eval(window_size, screen_props);

        let width = self.params.width.eval(window_size, screen_props);
        let height = self.params.height.eval(window_size, screen_props);

        let (x, y) = self.anchor.to_top_left(x, y, width, height);

        let image_offset_x = self.params.image_x.eval(window_size, screen_props);
        let image_offset_y = self.params.image_y.eval(window_size, screen_props);

        let trans_mat = self.transformation.eval(window_size, screen_props);

        frame.scene_mut().draw_shape_fill(
            Shape::Rectangle {
                a: (x, y).into(),
                w: width as f64,
                h: height as f64,
            },
            Brush::Image {
                image: &self.image,
                start: (x + image_offset_x, y + image_offset_y).into(),
                fit_mode: ImageFitMode::Exact { width, height },
                sampling: ImageSampling::Linear,
                edge_mode: (Extend::Pad, Extend::Pad),
                transform: None,
                alpha: Some(self.params.opacity as f32),
            },
            Some(self.transformation.eval(window_size, screen_props).into()),
            None,
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
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transformation.clone()
    }

    fn contains(&self, x: Size, y: Size, window: &Window) -> bool {
        let window_state = window.state.lock().unwrap();
        let window_size = window_state.size;
        let screen_props = window_state.physical_screen;

        let ix = self.params.x.eval(window_size, screen_props);
        let iy = self.params.y.eval(window_size, screen_props);
        let width = self.params.width.eval(window_size, screen_props);
        let height = self.params.height.eval(window_size, screen_props);

        let trans_mat = self.transformation.eval(window_size, screen_props);

        let x = x.eval(window_size, screen_props);
        let y = y.eval(window_size, screen_props);

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
