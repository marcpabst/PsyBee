use std::sync::{Arc, Mutex};

use cosmic_text::{
    Attrs, Buffer as CosmicBuffer, Family as CosmicFamily, Metrics as CosmicMetrics, Stretch as CosmicStretch,
    Style as CosmicStyle, Weight as CosmicWeight,
};
use image::DynamicImage;

use super::scenes::{DynamicScene, Scene};
use crate::{
    bitmaps::DynamicBitmap,
    font::{DynamicFontFace, FontStyle, FontWidth},
    shapes::Point,
};

pub struct DynamicRenderer {
    backend: Box<dyn Renderer>,
}

impl DynamicRenderer {
    pub(crate) fn new(backend_renderer: Box<dyn Renderer>) -> Self {
        DynamicRenderer {
            backend: backend_renderer,
        }
    }
    pub fn render_to_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: &mut DynamicScene,
    ) {
        self.backend
            .render_to_texture(device, queue, texture, width, height, scene.inner().as_mut());
    }

    pub fn create_renderer_factory(&self) -> Box<dyn RendererFactory> {
        self.backend.create_renderer_factory()
    }

    pub fn create_scene(&self, width: u32, heigth: u32) -> DynamicScene {
        let scene = self.backend.create_scene(width, heigth);
        DynamicScene::new(scene)
    }

    pub fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap {
        self.backend.create_bitmap(data)
    }

    pub fn create_bitmap_from_path(&self, path: &str) -> DynamicBitmap {
        self.backend.create_bitmap_from_path(path)
    }
}

pub trait Renderer {
    fn render_to_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: &mut dyn Scene,
    );

    fn create_scene(&self, width: u32, heigth: u32) -> Box<dyn Scene>;

    fn load_font_face(
        &mut self,
        face_info: &cosmic_text::fontdb::FaceInfo,
        font_data: &[u8],
        index: usize,
    ) -> DynamicFontFace;

    fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap;

    fn create_bitmap_from_path(&self, path: &str) -> DynamicBitmap {
        let image = image::open(path).unwrap();
        self.create_bitmap(image)
    }

    fn create_renderer_factory(&self) -> Box<dyn RendererFactory>;
}

pub trait RendererFactory: Send + Sync + std::fmt::Debug {
    fn create_renderer(
        &self,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> DynamicRenderer;

    fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap;

    fn create_bitmap_from_path(&self, path: &str) -> DynamicBitmap {
        let image = image::open(path).unwrap();
        self.create_bitmap(image)
    }

    fn create_font_face(&self, font_data: &[u8], index: u32) -> DynamicFontFace;

    // fn create_formatted_text(
    //     &mut self,
    //     font_manager: Arc<Mutex<cosmic_text::FontSystem>>,
    //     position: Point,
    //     text: &str,
    //     font_family: Option<&str>,
    //     font_weight: Option<u16>,
    //     font_stretch: Option<FontWidth>,
    //     font_style: Option<FontStyle>,
    //     font_size: f32,
    // ) -> FormatedText {
    //     let families = font_family
    //         .map(|f| CosmicFamily::Name(f))
    //         .unwrap_or(CosmicFamily::Name("Arial"));

    //     let weight = font_weight.map(|w| CosmicWeight(w)).unwrap_or(CosmicWeight(400));
    //     // let stretch = font_stretch.unwrap_or(FontWidth::Normal);
    //     // let style = font_style.unwrap_or(FontStyle::Normal);

    //     // Attributes indicate what font to choose
    //     let attrs = Attrs::new();
    //     let attrs = attrs.family(families);
    //     let attrs = attrs.weight(weight);
    //     let attrs = attrs.stretch(CosmicStretch::Normal);
    //     let attrs = attrs.style(CosmicStyle::Normal);

    //     let query = cosmic_text::fontdb::Query {
    //         families: &[families],
    //         weight: weight,
    //         stretch: CosmicStretch::Normal,
    //         style: CosmicStyle::Normal,
    //     };

    //     let mut font_manager_mg = font_manager.lock().unwrap();

    //     let cosmic_font_id = font_manager_mg.db().query(&query).unwrap();
    //     let cosmic_font = font_manager_mg.get_font(cosmic_font_id).unwrap();
    //     let comic_metrics = CosmicMetrics::new(font_size, font_size);
    //     // let comic_metrics = CosmicMetrics::new(14.0, 20.0);

    //     let face_info = font_manager_mg.db().face(cosmic_font_id).unwrap();
    //     let font_data = cosmic_font.data();
    //     let font_index = face_info.index;

    //     assert!(attrs.matches(&face_info));

    //     let font = self.create_font_face(font_data, font_index);
    //     let mut cosmic_buffer = CosmicBuffer::new(&mut font_manager_mg, comic_metrics);

    //     // Set a size for the text buffer, in pixels
    //     cosmic_buffer.set_size(&mut font_manager_mg, None, None);

    //     // Add some text!
    //     cosmic_buffer.set_text(&mut font_manager_mg, text, attrs, cosmic_text::Shaping::Advanced);

    //     // Perform shaping
    //     cosmic_buffer.shape_until_scroll(&mut font_manager_mg, true);

    //     // try to load the font
    //     FormatedText {
    //         cosmic_buffer: cosmic_buffer,
    //         renderer_font: font,
    //         cosmic_font: cosmic_font,
    //         comic_metrics: comic_metrics,
    //         position: position,
    //         alignment: Alignment::Center,
    //         vertical_alignment: VerticalAlignment::Middle,
    //         font_manager: font_manager.clone(),
    //     }
    // }

    fn cloned(&self) -> Box<dyn RendererFactory>;
}
