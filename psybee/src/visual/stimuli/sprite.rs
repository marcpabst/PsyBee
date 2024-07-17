use super::PyStimulus;
use image::{DynamicImage, GenericImageView};
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
pub struct SpriteStimulus {
    id: uuid::Uuid,

    origin: (Size, Size),
    transformation: Transformation2D,
    visible: bool,

    textures: Vec<Texture>,
    texture_index: usize,

    fps: Option<f32>,
    repeat: Option<u32>,

    start_time: std::time::Instant,

    pub width: Size,
    pub height: Size,
}

impl SpriteStimulus {
    pub fn new_from_images(
        width: Size,
        height: Size,
        images: Vec<image::DynamicImage>,
        fps: Option<f32>,
        repeat: Option<u32>,
    ) -> Self {
        let textures = images
            .into_iter()
            .map(|image| -> _ { Texture::from_image(image, TextureFormat::Srgba8U) })
            .collect();

        Self {
            id: Uuid::new_v4(),
            origin: (Size::Pixels(0.0), Size::Pixels(0.0)),
            transformation: Transformation2D::Identity(),
            visible: true,
            textures,
            texture_index: 0,
            fps,
            repeat,
            start_time: std::time::Instant::now(),
            width,
            height,
        }
    }

    pub fn new_from_paths(width: Size, height: Size, paths: Vec<&str>, fps: Option<f32>, repeat: Option<u32>) -> Self {
        let images = paths
            .into_iter()
            .map(|path| -> _ { image::open(path) })
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to open the images");

        Self::new_from_images(width, height, images, fps, repeat)
    }

    pub fn new_from_spritesheet(
        width: Size,
        height: Size,
        path: &str,
        num_sprites_x: u32,
        num_sprites_y: u32,
        fps: Option<f32>,
        repeat: Option<u32>,
    ) -> Self {
        let image = image::open(path).expect("Failed to open the sprite sheet");
        let (img_width, img_height) = image.dimensions();

        // check that the sprite sheet is divisible by the sprite size
        if img_width % num_sprites_x != 0 || img_height % num_sprites_y != 0 {
            panic!("The sprite sheet is not divisible by the sprite size");
        }

        // calculate the sprite size
        let sprite_width = img_width / num_sprites_x;
        let sprite_height = img_height / num_sprites_y;

        println!("Sprite size: {}x{}", sprite_width, sprite_height);

        // split the image into sprites
        let mut images: Vec<image::DynamicImage> = Vec::new();
        for y in (0..img_height).step_by(sprite_height as usize) {
            for x in (0..img_width).step_by(sprite_width as usize) {
                let sprite = image.view(x, y, sprite_width, sprite_height).to_image();
                images.push(DynamicImage::ImageRgba8(sprite));
            }
        }

        Self::new_from_images(width, height, images, fps, repeat)
    }

    pub fn next_image(&mut self) {
        self.texture_index = (self.texture_index + 1) % self.textures.len();
    }

    pub fn reset(&mut self) {
        self.start_time = std::time::Instant::now();
    }
}

impl Stimulus for SpriteStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&self, window: &Window) -> Vec<Renderable> {
        // find current index
        // if fps is set, calculate the index based on the time
        let mut index = self.texture_index;
        if let Some(fps) = self.fps {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let frames = elapsed * fps;
            index = frames as usize;
        }

        if let Some(repeat) = self.repeat {
            // if repeat is set, make sure that the new index is within the maximum index
            // if not, set index to the last image
            if index >= self.textures.len() * repeat as usize {
                index = self.textures.len() * repeat as usize - 1;
            }
        }

        // calculate the current index by wrapping around the number of images
        let wrapped_index = index % self.textures.len();

        // create the material
        let tmaterial = TextureMaterial {
            texture: self.textures[wrapped_index].clone(),
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

    fn set_transformation(&mut self, transformation: Transformation2D) {
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "SpriteStimulus", extends=PyStimulus)]
pub struct PySpriteStimulus();

#[pymethods]
impl PySpriteStimulus {
    #[new]
    fn __new__(
        width: Size,
        height: Size,
        sprite_sheet_path: &str,
        num_sprites_x: u32,
        num_sprites_y: u32,
        fps: Option<f32>,
        repeat: Option<u32>,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Box::new(SpriteStimulus::new_from_spritesheet(
                width,
                height,
                sprite_sheet_path,
                num_sprites_x,
                num_sprites_y,
                fps,
                repeat,
            ))),
        )
    }

    #[pyo3(name = "next_image")]
    fn py_next_image(slf: PyRefMut<'_, Self>) {
        slf.into_super()
            .0
            .downcast_mut::<SpriteStimulus>()
            .expect("Failed to downcast to ColourStimulus")
            .next_image()
    }

    #[pyo3(name = "reset")]
    fn py_reset(slf: PyRefMut<'_, Self>) {
        slf.into_super()
            .0
            .downcast_mut::<SpriteStimulus>()
            .expect("Failed to downcast to ColourStimulus")
            .reset()
    }
}
