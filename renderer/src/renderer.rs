use super::scenes::{DynamicScene, Scene};
use super::{skia_backend, Backend};
use crate::bitmaps::DynamicBitmap;
use crate::shapes::Point;
use crate::text::{Alignment, DynamicFontFace, FontStyle, FontWidth, FormatedText, VerticalAlignment};
use cosmic_text::{Attrs, Family, Weight};
use image::DynamicImage;
use vello::RendererOptions;
use wgpu::Backends;

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

    pub fn create_text(
        &self,
        fm: &mut cosmic_text::FontSystem,
        position: Point,
        text: &str,
        font_family: Option<&str>,
        font_weight: Option<u16>,
        font_stretch: Option<FontWidth>,
        font_style: Option<FontStyle>,
        size: f32,
    ) -> FormatedText {
        let families = font_family
            .map(|f| Family::Name(f))
            .unwrap_or(Family::Name("Arial"));
        let weight = font_weight.map(|w| Weight(w)).unwrap_or(Weight(400));
        let stretch = font_stretch.unwrap_or(FontWidth::Normal);
        let style = font_style.unwrap_or(FontStyle::Normal);
        // load the cosmic font
        let query = cosmic_text::fontdb::Query {
            families: &[families],
            weight: weight,
            stretch: Default::default(),
            style: Default::default(),
        };
        let cosmic_font_id = fm.db().query(&query).unwrap();
        let cosmic_font = fm.get_font(cosmic_font_id).unwrap();
        let face_info = fm.db().face(cosmic_font_id).unwrap();

        // try to load the font
        let font = self.0.load_font_face(face_info);
        FormatedText {
            text: text.to_string(),
            font: font,
            cosmic_font,
            size,
            weight: 0.0,
            style: FontStyle::Normal,
            position: position,
            alignment: Alignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
        }
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

    fn load_font_face(&self, face_info: &cosmic_text::fontdb::FaceInfo) -> DynamicFontFace;
}
