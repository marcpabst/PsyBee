use super::PyStimulus;
use pyo3::prelude::*;
use uuid::Uuid;

use super::Stimulus;
use crate::{
    renderer::{
        material::{TextureFilter, TextureMaterial, TextureRepeat, TextureSize},
        texture::TextureFormat,
        Geom, Material, Point2D, Primitive, Renderable, TessellationOptions, Texture,
    },
    visual::{
        geometry::{Size, Transformation2D},
        window::Window,
    },
};

#[derive(Clone, Debug)]
pub struct ImageStimulus {
    id: uuid::Uuid,

    origin: (Size, Size),
    transformation: Transformation2D,
    visible: bool,

    texture: Texture,

    pub width: Size,
    pub height: Size,
}

impl Stimulus for ImageStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&self, window: &Window) -> Vec<Renderable> {
        // create the material

        let tmaterial = TextureMaterial {
            texture: self.texture.clone(),
            size_x: TextureSize::Relative(1.0),
            size_y: TextureSize::Relative(1.0),
            repeat_x: TextureRepeat::Clamp,
            repeat_y: TextureRepeat::Clamp,
            filter: TextureFilter::Linear,
        };

        let material = Material::Texture(tmaterial);

        let trans_mat = self
            .transformation
            .to_transformation_matrix(&window.physical_properties);

        let width_px = self.width.eval(&window.physical_properties);
        let height_px = self.height.eval(&window.physical_properties);

        // create the drawables
        let patch_geom = Geom::new(
            Primitive::Rectangle {
                a: Point2D { x: 0.0, y: 0.0 },
                b: Point2D {
                    x: width_px,
                    y: height_px,
                },
            },
            material,
            Some(trans_mat.into()),
            None,
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

    fn set_transformation(&mut self, transformation: Transformation2D) {
        self.transformation = transformation;
    }

    fn transformation(&self) -> Transformation2D {
        self.transformation.clone()
    }
}

impl ImageStimulus {
    pub fn new(width: Size, height: Size, image: image::DynamicImage) -> Self {
        let texture = Texture::from_image(image, TextureFormat::Srgba8U);

        Self {
            id: Uuid::new_v4(),
            origin: (Size::Pixels(0.0), Size::Pixels(0.0)),
            transformation: Transformation2D::Identity(),
            visible: true,
            texture,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "ImageStimulus", extends=PyStimulus)]
pub struct PyImageStimulus();

#[pymethods]
impl PyImageStimulus {
    #[new]
    fn __new__(width: Size, height: Size, image_path: String) -> (Self, PyStimulus) {
        let image = image::open(image_path).unwrap();
        (Self(), PyStimulus(Box::new(ImageStimulus::new(width, height, image))))
    }
}
