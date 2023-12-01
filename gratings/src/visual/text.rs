use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    CommandEncoder, Device, MultisampleState, Queue, RenderPass, SurfaceConfiguration,
    TextureFormat,
};

use crate::visual::Renderable;
use crate::visual::Window;

pub struct TextStimulus {
    pub text: Arc<Mutex<String>>,
    text_atlas: Arc<Mutex<TextAtlas>>,
    text_renderer: Arc<Mutex<TextRenderer>>,
    font_system: Arc<Mutex<FontSystem>>,
    text_buffer: Arc<Mutex<Buffer>>,
    text_cache: Arc<Mutex<SwashCache>>,
}

impl Clone for TextStimulus {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            text_atlas: self.text_atlas.clone(),
            text_renderer: self.text_renderer.clone(),
            font_system: self.font_system.clone(),
            text_buffer: self.text_buffer.clone(),
            text_cache: self.text_cache.clone(),
        }
    }
}

impl Renderable for TextStimulus {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> () {
        self.text_renderer
            .lock()
            .unwrap()
            .prepare(
                device,
                queue,
                &mut self.font_system.lock().unwrap(),
                &mut self.text_atlas.lock().unwrap(),
                Resolution {
                    width: config.width,
                    height: config.height,
                },
                [TextArea {
                    buffer: &self.text_buffer.lock().unwrap(),
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
                &mut self.text_cache.lock().unwrap(),
            )
            .unwrap();
    }
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // Lock and dereference to get to the inner data
        let text_renderer = self.text_renderer.lock().unwrap();
        let atlas = self.text_atlas.lock().unwrap();
        {
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            text_renderer.render(&atlas, &mut rpass).unwrap();
        }
    }
}

impl TextStimulus {
    pub fn new(window: &Window, text: String) -> Self {
        let binding = window.device.clone();
        let device = &binding.lock().unwrap();
        let binding = window.queue.clone();
        let queue = &binding.lock().unwrap();
        let binding = window.adapter.clone();
        let adapter = &binding.lock().unwrap();

        let swapchain_capabilities = window
            .surface
            .clone()
            .lock()
            .unwrap()
            .get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let (width, height) = (800, 600);
        let scale_factor = 1.0;
        // Set up text renderer
        let mut font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, swapchain_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
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
            text: Arc::new(Mutex::new(text)),
            text_atlas: Arc::new(Mutex::new(atlas)),
            text_renderer: Arc::new(Mutex::new(text_renderer)),
            font_system: Arc::new(Mutex::new(font_system)),
            text_buffer: Arc::new(Mutex::new(buffer)),
            text_cache: Arc::new(Mutex::new(cache)),
        }
    }
}
