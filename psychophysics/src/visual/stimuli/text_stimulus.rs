use async_lock::Mutex;
use async_trait::async_trait;
use glyphon::cosmic_text::Align;
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, Stretch, Style,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Weight,
};
use palette::white_point::D65;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::visual::color::RawRgba;

use crate::visual::window::Window;

use wgpu::{Device, MultisampleState, Queue, SurfaceConfiguration};

use crate::visual::geometry::{Rectangle, Size, ToPixels};
use crate::visual::Renderable;

use crate::visual::color::ColorFormat;

/// A text stimulus.
pub struct TextStimulus {
    config: Arc<Mutex<TextStimulusConfig>>,
    text_atlas: Arc<Mutex<TextAtlas>>,
    text_renderer: Arc<Mutex<TextRenderer>>,
    font_system: Arc<Mutex<FontSystem>>,
    text_buffer: Arc<Mutex<Buffer>>,
    text_cache: Arc<Mutex<SwashCache>>,
    color_format: ColorFormat,
}

pub struct TextStimulusConfig {
    // the text to display
    pub text: String,
    // the font size
    pub font_size: Size,
    // the line height
    pub line_height: Size,
    // the weight of the font
    pub font_weight: Weight,
    // the style of the font
    pub font_style: Style,
    // the stretch of the font
    pub font_width: Stretch,
    // the bounds of the text
    pub bounds: Rectangle,
    // the color of the text
    pub color: RawRgba,
}

// default values for the text stimulus
impl Default for TextStimulusConfig {
    fn default() -> Self {
        Self {
            text: String::from(""),
            font_size: Size::Points(62.0),
            line_height: Size::Points(62.0),
            bounds: Rectangle::new(0.0, 0.0, 500.0, 500.0),
            font_weight: Weight::NORMAL,
            font_style: Style::Normal,
            font_width: Stretch::Normal,
            color: RawRgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into(),
        }
    }
}

impl Clone for TextStimulus {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            text_atlas: self.text_atlas.clone(),
            text_renderer: self.text_renderer.clone(),
            font_system: self.font_system.clone(),
            text_buffer: self.text_buffer.clone(),
            text_cache: self.text_cache.clone(),
            color_format: self.color_format.clone(),
        }
    }
}

#[async_trait(?Send)]
impl Renderable for TextStimulus {
    async fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        _view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &Window,
    ) -> () {
        let conf = self.config.lock_blocking();

        {
            // update the text buffer
            let mut buffer = self.text_buffer.lock_blocking();
            let mut font_system = self.font_system.lock_blocking();
            buffer.set_text(
                &mut font_system,
                &conf.text,
                Attrs::new()
                    .family(Family::SansSerif)
                    .weight(conf.font_weight)
                    .style(conf.font_style)
                    .stretch(conf.font_width),
                Shaping::Advanced,
            );
            buffer.lines[0].set_align(Some(Align::Center));
            buffer.shape_until_scroll(&mut font_system);
        }

        // convert bounds to pixels
        let screen_width_mm = window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm = window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = config.width;
        let screen_height_px = config.height;
        let bounds_px = conf.bounds.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        self.text_renderer
            .lock_blocking()
            .prepare(
                device,
                queue,
                &mut self.font_system.lock_blocking(),
                &mut self.text_atlas.lock_blocking(),
                Resolution {
                    width: screen_width_px,
                    height: screen_height_px,
                },
                [TextArea {
                    buffer: &self.text_buffer.lock_blocking(),
                    left: (bounds_px[0] + config.width as f64 / 2.0) as f32,
                    top: (bounds_px[1] + config.height as f64 / 2.0) as f32,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: config.width as i32,
                        bottom: config.height as i32,
                    },
                    default_color: conf.color.into(),
                }],
                &mut self.text_cache.lock_blocking(),
            )
            .unwrap();
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        let text_renderer = self.text_renderer.lock_blocking();
        let atlas = self.text_atlas.lock_blocking();
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
    pub fn new(window: &Window, text: impl Into<String>, rect: Rectangle) -> Self {
        let window = window.clone();

        let config = TextStimulusConfig {
            text: text.into(),
            line_height: rect.height.clone(),
            bounds: rect,
            ..Default::default()
        };

        //window.clone().run_on_render_thread(|| async move {
        log::info!("Creating text stimulus");
        let gpu_state = window.gpu_state.read_blocking();
        let window_state = window.state.read_blocking();

        let device = &gpu_state.device;
        let queue = &gpu_state.queue;
        let sconfig = window_state.config.clone();

        let swapchain_format = wgpu::TextureFormat::Bgra8Unorm;

        let screen_width_mm = window.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm = window.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = sconfig.width;
        let screen_height_px = sconfig.height;

        let bounds_px = config.bounds.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        let font_size_px = config.font_size.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        let line_height_px = config.line_height.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        let width: f64 = bounds_px[2] as f64;
        let height: f64 = bounds_px[3] as f64;

        let scale_factor = 1.0;

        // Set up text renderer
        // load fonts
        let plex_fonts = vec![
            include_bytes!("./assets/IBMPlexSans-Regular.ttf").to_vec(),
            include_bytes!("./assets/IBMPlexSans-Bold.ttf").to_vec(),
            include_bytes!("./assets/IBMPlexSans-Italic.ttf").to_vec(),
            include_bytes!("./assets/IBMPlexSans-BoldItalic.ttf").to_vec(),
        ];

        let mut font_system = FontSystem::new();

        for plex_font in plex_fonts {
            font_system.db_mut().load_font_data(plex_font);
        }

        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, swapchain_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        let mut buffer = Buffer::new(
            &mut font_system,
            Metrics::new(font_size_px as f32, line_height_px as f32),
        );

        let physical_width = (width as f64 * scale_factor) as f32;
        let physical_height = (height as f64 * scale_factor) as f32;

        buffer.set_size(&mut font_system, physical_width, physical_height);

        buffer.set_text(
            &mut font_system,
            &config.text,
            Attrs::new().family(Family::Name("IBM Plex Sans")),
            Shaping::Advanced,
        );
        buffer.lines[0].set_align(Some(Align::Center));
        buffer.shape_until_scroll(&mut font_system);

        Self {
            config: Arc::new(Mutex::new(config)),
            text_atlas: Arc::new(Mutex::new(atlas)),
            text_renderer: Arc::new(Mutex::new(text_renderer)),
            font_system: Arc::new(Mutex::new(font_system)),
            text_buffer: Arc::new(Mutex::new(buffer)),
            text_cache: Arc::new(Mutex::new(cache)),
            color_format: window.color_format,
        }

        // })
    }

    pub fn set_color(&self, color: impl palette::IntoColor<palette::Xyza<D65, f32>>) {
        let color: palette::Xyza<D65, f32> = color.into_color();
        let color = self.color_format.convert_to_raw_rgba(color);
        let mut conf = self.config.lock_blocking();

        conf.color = color;
    }

    pub fn set_text(&self, text: String) {
        let mut conf = self.config.lock_blocking();
        conf.text = text;
    }
}
