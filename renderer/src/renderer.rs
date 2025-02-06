use image::DynamicImage;
use super::scenes::{DynamicScene, Scene};
use super::{skia_backend, Backend};
use vello::RendererOptions;
use wgpu::Backends;
use crate::bitmaps::DynamicBitmap;

pub struct DynamicRenderer(pub Box<dyn Renderer>);

impl DynamicRenderer {
    pub fn new(
        backend: Backend,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        match backend {
            Backend::Vello => {
                let renderer = vello::Renderer::new(
                    &device,
                    RendererOptions {
                        surface_format: Some(surface_format),
                        use_cpu: false,
                        antialiasing_support: vello::AaSupport::all(),
                        num_init_threads: std::num::NonZeroUsize::new(1),
                    },
                )
                .unwrap();
                Self(Box::new(renderer))
            }
            Backend::Skia => {
                let renderer = skia_backend::SkiaRenderer::new(width, height, device);
                Self(Box::new(renderer))
            }
        }
    }
    pub fn render_to_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: &mut DynamicScene,
    ) {
        self.0
            .render_to_texture(device, queue, texture, width, height, scene.0.as_mut());
    }

    pub fn create_scene(&self, width: u32, heigth: u32) -> DynamicScene {
        let scene = self.0.create_scene(width, heigth);
        DynamicScene(scene)
    }
    
    pub fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap {
        self.0.create_bitmap(data)
    }
}

pub trait Renderer {
    fn render_to_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: &mut dyn Scene,
    );

    fn create_scene(&self, width: u32, heigth: u32) -> Box<dyn Scene>;
    
    fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap;
}
