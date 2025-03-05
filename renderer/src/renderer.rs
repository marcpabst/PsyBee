use cosmic_text::{
    Attrs, Buffer as CosmicBuffer, Family as CosmicFamily, Metrics as CosmicMetrics, Stretch as CosmicStretch,
    Style as CosmicStyle, Weight as CosmicWeight,
};
use image::DynamicImage;

use super::{
    scenes::{DynamicScene, Scene},
    skia_backend, Backend,
};
use crate::{
    bitmaps::DynamicBitmap,
    shapes::Point,
    text::{Alignment, DynamicFontFace, FontStyle, FontWidth, FormatedText, VerticalAlignment},
};

pub struct DynamicRenderer {
    font_manager: cosmic_text::FontSystem,
    backend: Box<dyn Renderer>,
}

impl DynamicRenderer {
    pub(crate) fn new(
        backend_renderer: Box<dyn Renderer>,
        // backend: Backend,
        // adapter: &wgpu::Adapter,
        // device: &wgpu::Device,
        // queue: &wgpu::Queue,
        // surface_format: wgpu::TextureFormat,
        // width: u32,
        // height: u32,
    ) -> Self {
        // create a font system (=font manager)
        let empty_db = cosmic_text::fontdb::Database::new();
        let mut font_manager = cosmic_text::FontSystem::new_with_locale_and_db("en".to_string(), empty_db);

        // load Noto Sans
        let noto_sans_regular = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
        font_manager.db_mut().load_font_data(noto_sans_regular.to_vec());
        let noto_sans_bold = include_bytes!("../assets/fonts/NotoSans-Bold.ttf");
        font_manager.db_mut().load_font_data(noto_sans_bold.to_vec());
        let noto_sans_italic = include_bytes!("../assets/fonts/NotoSans-Italic.ttf");
        font_manager.db_mut().load_font_data(noto_sans_italic.to_vec());
        let noto_sans_bold_italic = include_bytes!("../assets/fonts/NotoSans-BoldItalic.ttf");
        font_manager.db_mut().load_font_data(noto_sans_bold_italic.to_vec());

        // let backend_renderer = match backend {
        //     Backend::Vello => {
        //         // let renderer = vello::Renderer::new(
        //         //     &device,
        //         //     RendererOptions {
        //         //         surface_format: Some(surface_format),
        //         //         use_cpu: false,
        //         //         antialiasing_support: vello::AaSupport::all(),
        //         //         num_init_threads: std::num::NonZeroUsize::new(1),
        //         //     },
        //         // )
        //         // .unwrap();
        //         todo!()
        //         // Box::new(renderer) as Box<dyn Renderer>
        //     }
        //     Backend::Skia => {
        //         let renderer = skia_backend::SkiaRenderer::new(width, height, adapter, device, queue);
        //         Box::new(renderer) as Box<dyn Renderer>
        //     }
        // };

        DynamicRenderer {
            font_manager: font_manager,
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

    pub fn create_text(
        &mut self,
        position: Point,
        text: &str,
        font_family: Option<&str>,
        font_weight: Option<u16>,
        font_stretch: Option<FontWidth>,
        font_style: Option<FontStyle>,
        font_size: f32,
    ) -> FormatedText {
        let families = font_family
            .map(|f| CosmicFamily::Name(f))
            .unwrap_or(CosmicFamily::Name("Arial"));

        let weight = font_weight.map(|w| CosmicWeight(w)).unwrap_or(CosmicWeight(400));
        // let stretch = font_stretch.unwrap_or(FontWidth::Normal);
        // let style = font_style.unwrap_or(FontStyle::Normal);

        // Attributes indicate what font to choose
        let attrs = Attrs::new();
        let attrs = attrs.family(families);
        let attrs = attrs.weight(weight);
        let attrs = attrs.stretch(CosmicStretch::Normal);
        let attrs = attrs.style(CosmicStyle::Normal);

        let query = cosmic_text::fontdb::Query {
            families: &[families],
            weight: weight,
            stretch: CosmicStretch::Normal,
            style: CosmicStyle::Normal,
        };

        let cosmic_font_id = self.font_manager.db().query(&query).unwrap();
        let cosmic_font = self.font_manager.get_font(cosmic_font_id).unwrap();
        let comic_metrics = CosmicMetrics::new(font_size, font_size);
        // let comic_metrics = CosmicMetrics::new(14.0, 20.0);

        let face_info = self.font_manager.db().face(cosmic_font_id).unwrap();
        let font_data = cosmic_font.data();
        let font_index = face_info.index;

        assert!(attrs.matches(&face_info));

        println!("font_data len: {:?}", font_data.len());

        let font = self.backend.load_font_face(face_info, &font_data, font_index as usize);

        let mut cosmic_buffer = CosmicBuffer::new(&mut self.font_manager, comic_metrics);

        // Set a size for the text buffer, in pixels
        cosmic_buffer.set_size(&mut self.font_manager, None, None);

        // Add some text!
        cosmic_buffer.set_text(&mut self.font_manager, text, attrs, cosmic_text::Shaping::Advanced);

        // Perform shaping as desired
        cosmic_buffer.shape_until_scroll(&mut self.font_manager, true);

        // try to load the font
        FormatedText {
            cosmic_buffer: cosmic_buffer,
            renderer_font: font,
            cosmic_font: cosmic_font,
            comic_metrics: comic_metrics,
            position: position,
            alignment: Alignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
        }
    }

    pub fn font_manager(&self) -> &cosmic_text::FontSystem {
        &self.font_manager
    }

    pub fn font_manager_mut(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.font_manager
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

    fn cloned(&self) -> Box<dyn RendererFactory>;
}
