use super::{
    super::geometry::{Size, ToVertices},
    super::pwindow::WindowHandle,
    base::{BaseStimulus, BaseStimulusPixelShader, ShapeStimulusParams},
};
use bytemuck::{Pod, Zeroable};
use futures_lite::future::block_on;
use half::f16;
use image;
use std::borrow::Cow;
use wgpu::{Device, ShaderModule};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ImageStimulusParams {
    s: f32,
}

// TODO: make this a macro
impl ShapeStimulusParams for ImageStimulusParams {}

pub struct ImageStimulusShader {
    shader: ShaderModule,
    image: image::DynamicImage,
}

pub type ImageStimulus<'a, G> =
    BaseStimulus<G, ImageStimulusShader, ImageStimulusParams>;

impl<G: ToVertices> ImageStimulus<'_, G> {
    /// Create a new image stimulus.
    pub fn new(
        window_handle: &WindowHandle,
        shape: G,
        image: image::DynamicImage,
    ) -> Self {
        let window = block_on(window_handle.get_window());
        let device = &window.device;

        let shader = ImageStimulusShader::new(&device, image);

        let params = ImageStimulusParams { s: 3.0 };

        drop(window); // this prevent a deadlock (argh, i'll have to refactor this)

        let texture_size = wgpu::Extent3d {
            width: shader.image.width(),
            height: shader.image.height(),
            depth_or_array_layers: 1,
        };

        let texture_data = shader.image.to_rgba8();
        // convert to Rgba16Float
        let texture_data: Vec<f16> = texture_data
            .iter()
            .map(|x| f16::from_f32(*x as f32 / 255.0))
            .collect();

        let out = BaseStimulus::create(
            window_handle,
            shader,
            shape,
            params,
            Some(texture_size),
        );

        // upload texture data
        out.set_texture(texture_data.as_slice());

        out
    }
}

impl ImageStimulusShader {
    pub fn new(device: &Device, image: image::DynamicImage) -> Self {
        let shader: ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shaders/image.wgsl"
                ))),
            });

        Self { shader, image }
    }
}

impl BaseStimulusPixelShader<ImageStimulusParams> for ImageStimulusShader {
    fn prepare(
        &self,
        params: &mut ImageStimulusParams,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) {
        // nothing to do here
    }
    fn get_shader(&self) -> &ShaderModule {
        &self.shader
    }
}
