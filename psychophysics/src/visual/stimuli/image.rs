use super::{
    super::geometry::{Rectangle, Size},
    super::pwindow::Window,
    base::{BaseStimulus, BaseStimulusImplementation, BaseStimulusParams},
};
use image;

pub(crate) struct ImageStimulusImplementation {
    image: image::DynamicImage,
    shape: Rectangle,
}

pub type ImageStimulus<'a> = BaseStimulus<ImageStimulusImplementation>;

impl ImageStimulus {
    /// Create a new image stimulus.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to which the stimulus will be added.
    /// * `image` - The image to be displayed.
    ///
    /// # Returns
    ///
    /// A new image stimulus.
    pub fn new(window: &Window, image: image::DynamicImage) -> Self {
        let window = window.clone();
        window.clone().run_on_render_thread(|| async move {
            BaseStimulus::create(
                &window.get_window_state().await.device,
                ImageStimulusImplementation::new(&device, image),
            )
        })
    }

    /// Set the rectangle used to display the image on the screen.
    ///
    /// # Arguments
    ///
    /// * `shape` - A rectangle that defines the position and size of the image.
    pub fn set_rectangle(&self, shape: Rectangle) {
        (self.stimulus_implementation.lock_blocking()).shape = shape;
    }
}

impl ImageStimulusImplementation {
    pub fn new(device: &Device, image: image::DynamicImage) -> Self {
        // by default, we create a rectangle that fills the screen
        let shape = Rectangle::new(
            -Size::Pixels(image.width() as f64 / 2.0),
            -Size::Pixels(image.height() as f64 / 2.0),
            Size::Pixels(image.width() as f64),
            Size::Pixels(image.height() as f64),
        );

        Self { image, shape }
    }
}

impl BaseStimulusImplementation for ImageStimulusImplementation {
    fn get_fragment_shader_code(&self) -> String {
        "
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(texture, texture_sampler, in.tex_coords);
        }
        "
        .to_string()
    }

    fn get_texture_data(&self) -> Option<(Vec<u8>)> {
        // convert from rgba to bgra
        let texture_data: Vec<u8> = self
            .image
            .to_rgba8()
            .chunks_exact(4)
            .flat_map(|chunk| {
                [
                    chunk[2], // r
                    chunk[1], // g
                    chunk[0], // b
                    chunk[3], // a
                ]
            })
            .collect();

        Some(texture_data)
    }

    fn get_texture_size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.image.width(),
            height: self.image.height(),
            depth_or_array_layers: 1,
        }
    }

    fn get_geometry(&self) -> Box<dyn crate::visual::geometry::ToVertices> {
        Box::new(self.shape.clone())
    }
}
