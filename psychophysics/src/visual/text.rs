use crate::utils::BlockingLock;
use async_lock::Mutex;
use futures_lite::future::block_on;
use glyphon::cosmic_text::Align;
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, Stretch,
    Style, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Weight,
};
use palette::white_point::D65;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::visual::color::RawRgba;

use crate::visual::pwindow::Window;

use wgpu::{Device, MultisampleState, Queue, SurfaceConfiguration};

use crate::visual::geometry::{Rectangle, Size};
use crate::visual::Renderable;

use super::color;

pub struct TextStimulus {
    config: Arc<Mutex<TextStimulusConfig>>,
    text_atlas: Arc<Mutex<TextAtlas>>,
    text_renderer: Arc<Mutex<TextRenderer>>,
    font_system: Arc<Mutex<FontSystem>>,
    text_buffer: Arc<Mutex<Buffer>>,
    text_cache: Arc<Mutex<SwashCache>>,
    color_format: color::ColorFormat,
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
            bounds: Rectangle::new(-250.0, -42.0, 500.0, 42.0),
            font_weight: Weight::NORMAL,
            font_style: Style::Normal,
            font_width: Stretch::Normal,
            color: RawRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
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

impl Renderable for TextStimulus {
    fn prepare(
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
        let screen_width_mm =
            window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = config.width as i32;
        let screen_height_px = config.height as i32;
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
                    width: config.width,
                    height: config.height,
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
    fn render(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> () {
        // Lock and dereference to get to the inner data
        let text_renderer = self.text_renderer.lock_blocking();
        let atlas = self.text_atlas.lock_blocking();
        {
            let mut rpass =
                enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            text_renderer.render(&atlas, &mut rpass).unwrap();
        }
    }
}

impl TextStimulus {
    pub fn new(window_handle: &Window, config: TextStimulusConfig) -> Self {
        let window = window_handle.get_window_state_blocking();
        let device = &window.device;
        let queue = &window.queue;
        let adapter = &window.adapter;
        let sconfig = window.config.clone();

        let swapchain_capabilities = window.surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let screen_width_mm =
            window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = sconfig.width as i32;
        let screen_height_px = sconfig.height as i32;
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
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &device,
            MultisampleState::default(),
            None,
        );
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
            color_format: window_handle.color_format,
        }
    }

    pub fn set_color(
        &mut self,
        color: impl palette::IntoColor<palette::Xyza<D65, f32>>,
    ) {
        let color: palette::Xyza<D65, f32> = color.into_color();
        log::info!("Setting color to {:?}", color);
        let color = self.color_format.convert_to_raw_rgba(color);
        log::info!("Setting color to {:?}", color);
        let mut conf = self.config.lock_blocking();
        conf.color = color;
    }

    pub fn set_text(&mut self, text: String) {
        let mut conf = self.config.lock_blocking();
        conf.text = text;
    }
}
