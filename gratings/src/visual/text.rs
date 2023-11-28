use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    Adapter, CommandEncoder, Device, MultisampleState, Queue, RenderPass, ShaderModule, Surface,
    SurfaceConfiguration, TextureFormat, TextureView,
};

use crate::visual::Renderable;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct TextStimulusParams {}

pub struct TextStimulus {
    pub text: String,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    font_system: FontSystem,
    text_buffer: Buffer,
    text_cache: SwashCache,
}

impl Renderable for TextStimulus {
    fn render<'pass>(&'pass self, _device: &mut Device, mut pass: &mut RenderPass<'pass>) -> () {
        self.text_renderer
            .render(&self.text_atlas, &mut pass)
            .unwrap();
    }
    fn update(
        &mut self,
        device: &mut Device,
        queue: &Queue,
        _encoder: &mut CommandEncoder,
        config: &SurfaceConfiguration,
    ) -> () {
        self.text_renderer
            .prepare(
                &device,
                &queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: config.width,
                    height: config.height,
                },
                [TextArea {
                    buffer: &self.text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                &mut self.text_cache,
            )
            .unwrap();
    }
}

impl TextStimulus {
    pub fn new(
        device: &Device,
        queue: &wgpu::Queue,
        text: String,
        swapchain_format: TextureFormat,
    ) -> Self {
        let (width, height) = (800, 600);
        let scale_factor = 1.0;
        // Set up text renderer
        let mut font_system = FontSystem::new();
        let mut cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
        let mut text_renderer =
            TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (width as f64 * scale_factor) as f32;
        let physical_height = (height as f64 * scale_factor) as f32;

        buffer.set_size(&mut font_system, physical_width, physical_height);
        buffer.set_text(
            &mut font_system,
            text.as_str(),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut font_system);

        Self {
            text: text,
            text_atlas: atlas,
            text_renderer: text_renderer,
            font_system: font_system,
            text_buffer: buffer,
            text_cache: cache,
        }
    }
}
