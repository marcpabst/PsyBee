use crate::brushes::{Extend, ImageColor};
use cosmic_text::fontdb::FaceInfo;
use image::{DynamicImage, GenericImageView};
use skrifa::raw::FileRef;
use std::any::Any;
use std::sync::Arc;
use vello::kurbo::PathEl;
use vello::peniko::color::{AlphaColor, ColorSpaceTag};
use vello::peniko::BlendMode as VelloBlendMode;
use vello::peniko::Compose as VelloCompose;
use vello::peniko::Mix as VelloMix;
use vello::RendererOptions;
use wgpu::util::DeviceExt;

use super::brushes::{Gradient, GradientKind};
use super::text::{Alignment, FormatedText, VerticalAlignment};
use super::{
    brushes::{Brush, ColorStop},
    colors::RGBA,
};
use crate::bitmaps::DynamicBitmap;
use crate::prelude::DynamicFontFace;
use crate::renderer::Renderer;
use crate::shapes::{Point, Shape};
use crate::styles::{BlendMode, FillStyle, StrokeStyle};
use crate::{affine::Affine, scenes::Scene};

pub struct VelloScene {
    /// The Vello scene.
    pub vello_scene: vello::Scene,
    /// The global transform.
    pub global_transform: Affine,
    /// array of
    pub gpu_images: Vec<(vello::peniko::Image, wgpu::TexelCopyTextureInfoBase<wgpu::Texture>)>,
    /// The width of the scene in pixels.
    pub width: u32,
    /// The height of the scene in pixels.
    pub height: u32,
}

pub struct VelloRenderer {
    /// The vello renderer struct
    pub renderer: vello::Renderer,
    // /// An optional wgpu render pipeline used for rendering the texture that vello produces
    // pub render_pipeline: wgpu::RenderPipeline,
    // /// The texture that vello will render to
    // pub texture: wgpu::Texture,
    // /// Uniform buffer for gamma correction
    // pub gamma_buffer: wgpu::Buffer,
    // /// The bind group
    // pub bind_group: wgpu::BindGroup,
}

impl VelloRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
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

        // // create a render pipeline
        // let render_pipeline = Self::create_render_pipelie(device, surface_format);
        // let texture = Self::create_texture(device, width, height);
        // let gamma_buffer = Self::create_uniform_buffer(device);
        // let bind_group = Self::create_bind_group(device, &texture);

        Self {
            renderer,
            // render_pipeline,
            // texture,
            // gamma_buffer,
            // bind_group,
        }
    }
}

impl Renderer for vello::Renderer {
    /// Render the scene to a WGPU surface.
    // fn render_to_surface(
    //     &mut self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     surface: &wgpu::SurfaceTexture,
    //     scene: &impl Scene,
    // ) {
    //     // try to downcast the scene to a VelloScene
    //     let scene = scene
    //         .as_any()
    //         .downcast_ref::<VelloScene>()
    //         .expect("Failed to downcast scene to VelloScene");
    //     let vello_scene = &scene.backend.vello_scene;
    //     let render_params = vello::RenderParams {
    //         base_color: scene.background_color.into(),
    //         width: surface.texture.width(),
    //         height: surface.texture.height(),
    //         antialiasing_method: vello::AaConfig::Msaa16,
    //     };
    //     // (interim) replace the images with GPU textures.
    //     for (image, wgpu_texture) in &scene.backend.gpu_images {
    //         self.renderer.override_image(image, Some(wgpu_texture.clone()));
    //     }
    //     self.renderer
    //         .render_to_surface(device, queue, vello_scene, surface, &render_params)
    //         .expect("Failed to render to surface");
    // }

    /// Render the scene to a WGPU texture.
    fn render_to_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: &mut dyn Scene,
    ) {
        // try to downcast the scene to a VelloScene
        let vello_scene = &scene
            .as_any()
            .downcast_ref::<VelloScene>()
            .expect("Incorrect scene type. You can only use VelloScene with VelloRenderer");

        let render_params = vello::RenderParams {
            base_color: AlphaColor::from_rgba8(0, 0, 0, 0),
            width: width,
            height: height,
            antialiasing_method: vello::AaConfig::Msaa16,
        };

        // (interim) replace the images with GPU textures.
        // for (image, wgpu_texture) in &vello_scene.gpu_images {
        //     self.override_image(image, Some(wgpu_texture.clone()));
        // }

        // self.render_to_texture(
        //     device,
        //     queue,
        //     &vello_scene.vello_scene,
        //     &texture.create_view(&Default::default()),
        //     &render_params,
        // )
        // .expect("Failed to render to texture");
    }

    fn create_scene(&self, width: u32, heigth: u32) -> Box<dyn Scene> {
        Box::new(VelloScene::new(width, heigth))
    }

    fn create_bitmap(&self, data: DynamicImage) -> DynamicBitmap {
        todo!()
    }

    fn load_font_face(
        &mut self,
        face_info: &cosmic_text::fontdb::FaceInfo,
        font_data: &[u8],
        index: usize,
    ) -> DynamicFontFace {
        todo!()
    }
}

// /// Render the scene to a WGPU surface but sets up its own render pass.
// pub fn render_to_surface2(
//     &mut self,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     surface: &wgpu::SurfaceTexture,
//     scene: &Scene<VelloBackend>,
// ) {
//     // create texture view
//     let texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
//
//     // render the scene
//     self.render_to_texture(
//         device,
//         queue,
//         &texture_view,
//         surface.texture.width(),
//         surface.texture.height(),
//         scene,
//     );
//
//     // create a new render pass
//     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//         label: Some("Render Encoder"),
//     });
//
//     let surface_texture_view = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
//
//     {
//         // bind the render pass
//         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: Some("Render Pass"),
//             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                 view: &surface_texture_view,
//                 resolve_target: None,
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(wgpu::Color::RED),
//                     store: wgpu::StoreOp::Store,
//                 },
//             })],
//             depth_stencil_attachment: None,
//             timestamp_writes: None,
//             occlusion_query_set: None,
//         });
//
//         // bind the render pipeline
//         render_pass.set_pipeline(&self.render_pipeline);
//         // bind the bind group
//         render_pass.set_bind_group(0, &self.bind_group, &[]);
//         // draw the quad
//         render_pass.draw(0..6, 0..1);
//     }
//
//     // submit the render pass
//     queue.submit(Some(encoder.finish()));
// }
//
//
// }

impl VelloScene {
    /// Create a new Vello backend.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            vello_scene: vello::Scene::new(),
            global_transform: Affine::translate(width as f64 / 2.0, height as f64 / 2.0),
            gpu_images: Vec::new(),
            width,
            height,
        }
    }
}

macro_rules! dispatch_to_fun {
    ($value:expr, $fun:expr, $enum_name:ident { $($variant:ident),* }) => {
        match $value {
            $(
                $enum_name::$variant(ref inner) => $fun(inner),
            )*
        }
    };
}

impl Scene for VelloScene {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_background_color(&mut self, color: RGBA) {
        todo!()
    }

    fn set_width(&mut self, width: u32) {
        todo!()
    }

    fn set_height(&mut self, height: u32) {
        todo!()
    }

    fn background_color(&self) -> RGBA {
        todo!()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn start_layer(
        &mut self,
        composite_mode: BlendMode,
        clip: Shape,
        clip_transform: Option<Affine>,
        layer_transform: Option<Affine>,
        alpha: f32,
    ) {
        // error if a layer transform is provided
        if layer_transform.is_some() {
            todo!();
        }
        let clip_shape: VelloShape = clip.into();
        let global_transform = self.global_transform;
        let clip_transform = if let Some(clip_transform) = clip_transform {
            (global_transform * clip_transform)
        } else {
            global_transform
        };

        dispatch_to_fun!(
            clip_shape,
            |shape| {
                self.vello_scene
                    .push_layer(composite_mode, alpha, clip_transform.into(), shape);
            },
            VelloShape {
                Rectangle,
                RoundedRect,
                Circle,
                Ellipse,
                Line
            }
        );
    }

    fn end_layer(&mut self) {
        self.vello_scene.pop_layer();
    }

    fn draw_shape_fill(
        &mut self,
        shape: Shape,
        brush: Brush,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        let vello_shape = shape.into();
        dispatch_to_fun!(
            vello_shape,
            |shape| {
                self.vello_scene.fill(
                    FillStyle::NonZero.into(),
                    transform.unwrap_or(Affine::identity()).into(),
                    &brush.as_brush_or_brushref(),
                    None,
                    &shape,
                );
            },
            VelloShape {
                Rectangle,
                RoundedRect,
                Circle,
                Ellipse,
                Line
            }
        );
    }

    fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        // TODO: if a blend mode is provided, we need to push a new layer

        let vello_shape = shape.into();
        dispatch_to_fun!(
            vello_shape,
            |shape| {
                self.vello_scene.stroke(
                    &style.into(),
                    transform.unwrap_or(Affine::identity()).into(),
                    &brush.as_brush_or_brushref(),
                    None,
                    &shape,
                );
            },
            VelloShape {
                Rectangle,
                RoundedRect,
                Circle,
                Ellipse,
                Line
            }
        );
    }

    fn draw_glyphs(
        &mut self,
        position: Point,
        glyphs: &[crate::prelude::Glyph],
        font_face: &DynamicFontFace,
        font_size: f32,
        brush: Brush,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        todo!()
    }
}
//
// // Textures
// impl Image {
//     // /// Create a new texture from an image::DynamicImage.
//     // pub fn new(image: &image::DynamicImage, color_space: ImageColor) -> Self {
//     //     let data = Arc::new(image.clone().into_rgba8().into_vec());
//     //
//     //     return Self {
//     //         gpu_texture: None,
//     //         data,
//     //         width: image.width(),
//     //         height: image.height(),
//     //         color_space,
//     //     };
//     // }
//
//     /// Move the texture to the GPU.
//     // pub fn to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
//     //     if self.gpu_texture.is_some() {
//     //         return;
//     //     }
//     //     println!("Creating texture with color space: {:?}", self.color_space);
//     //     let data = &self.data;
//     //     // create a new wgpu texture
//     //     let wgpu_tetxure = device.create_texture(&wgpu::TextureDescriptor {
//     //         size: wgpu::Extent3d {
//     //             width: self.width,
//     //             height: self.height,
//     //             depth_or_array_layers: 1,
//     //         },
//     //         mip_level_count: 1,
//     //         sample_count: 1,
//     //         dimension: wgpu::TextureDimension::D2,
//     //         format: if self.color_space == ImageColor::LinearRGB {
//     //             wgpu::TextureFormat::Rgba8Unorm
//     //         } else {
//     //             wgpu::TextureFormat::Rgba8UnormSrgb
//     //         },
//     //         usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
//     //         label: None,
//     //         view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
//     //     });
//     //
//     //     // write the image to the texture
//     //     queue.write_texture(
//     //         wgpu::ImageCopyTexture {
//     //             texture: &wgpu_tetxure,
//     //             mip_level: 0,
//     //             origin: wgpu::Origin3d::ZERO,
//     //             aspect: wgpu::TextureAspect::All,
//     //         },
//     //         data.as_ref(),
//     //         wgpu::ImageDataLayout {
//     //             offset: 0,
//     //             bytes_per_row: Some(4 * self.width),
//     //             rows_per_image: Some(self.height),
//     //         },
//     //         wgpu::Extent3d {
//     //             width: self.width,
//     //             height: self.height,
//     //             depth_or_array_layers: 1,
//     //         },
//     //     );
//     //
//     //     self.gpu_texture = Some(Arc::new(wgpu_tetxure));
//     // }
// }

#[derive(Debug, Clone)]
pub enum VelloShape {
    Rectangle(vello::kurbo::Rect),
    RoundedRect(vello::kurbo::RoundedRect),
    Circle(vello::kurbo::Circle),
    Ellipse(vello::kurbo::Ellipse),
    Line(vello::kurbo::Line),
}

// allow converting Shape enum to VelloShape
impl Into<VelloShape> for Shape {
    fn into(self) -> VelloShape {
        match self {
            Shape::Rectangle { a, b } => VelloShape::Rectangle(vello::kurbo::Rect::new(a.x, a.y, b.x, b.y)),
            Shape::RoundedRectangle { a, b, radius } => {
                VelloShape::RoundedRect(vello::kurbo::RoundedRect::new(a.x, a.y, b.x, b.y, radius))
            }
            Shape::Circle { center, radius } => VelloShape::Circle(vello::kurbo::Circle::new(center, radius)),
            Shape::Line { start, end } => VelloShape::Line(vello::kurbo::Line::new(start, end)),
            Shape::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => VelloShape::Ellipse(vello::kurbo::Ellipse::new(
                center,
                (radius_x, radius_y),
                rotation.to_radians(),
            )),
        }
    }
}

// Point2D
impl From<Point> for vello::kurbo::Point {
    fn from(point: Point) -> Self {
        vello::kurbo::Point::new(point.x, point.y)
    }
}

// Affine
impl From<Affine> for vello::kurbo::Affine {
    fn from(affine: Affine) -> Self {
        vello::kurbo::Affine::new(affine.0)
    }
}

// FillStyle
impl From<FillStyle> for vello::peniko::Fill {
    fn from(style: FillStyle) -> Self {
        match style {
            FillStyle::NonZero => vello::peniko::Fill::NonZero,

            FillStyle::EvenOdd => vello::peniko::Fill::EvenOdd,
        }
    }
}

// StrokeStyle
impl From<StrokeStyle> for vello::kurbo::Stroke {
    fn from(style: StrokeStyle) -> Self {
        vello::kurbo::Stroke {
            width: style.width,
            ..Default::default()
        }
    }
}

// BrushRef (this needs to be refactored)
impl<'a> Brush<'_> {
    fn as_brush_or_brushref(&'a self) -> VelloBrushOrBrushRef<'a> {
        match self {
            Brush::Image { .. } => {
                todo!()
            }
            Brush::Solid(rgba) => VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Solid(rgba.clone().into())),
            Brush::Gradient(gradient) => {
                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Gradient(gradient.clone().into()))
            }
            _ => todo!(),
        }
    }
}

/// A brush or a brush reference
pub enum VelloBrushOrBrushRef<'a> {
    /// A brush brush
    Brush(vello::peniko::Brush),
    /// A brush reference
    BrushRef(vello::peniko::BrushRef<'a>),
}

// allow VelloBrushOrBrushRef to BrushRef
impl<'a> From<&'a VelloBrushOrBrushRef<'a>> for vello::peniko::BrushRef<'a> {
    fn from(brush: &'a VelloBrushOrBrushRef<'a>) -> Self {
        match brush {
            VelloBrushOrBrushRef::BrushRef(brush_ref) => brush_ref.clone(),
            VelloBrushOrBrushRef::Brush(brush) => brush.into(),
        }
    }
}

// allow to get peniko::Image from VelloBrushOrBrushRef
impl<'a> TryFrom<&'a VelloBrushOrBrushRef<'a>> for vello::peniko::Image {
    type Error = &'static str;

    fn try_from(brush: &'a VelloBrushOrBrushRef<'a>) -> Result<Self, Self::Error> {
        match brush {
            VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Image(image)) => Ok(image.clone()),
            _ => Err("Not an image brush"),
        }
    }
}

// Colors
impl From<RGBA> for vello::peniko::Color {
    fn from(color: RGBA) -> Self {
        vello::peniko::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        )
    }
}

impl From<RGBA> for vello::peniko::color::DynamicColor {
    fn from(color: RGBA) -> Self {
        let ac = vello::peniko::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        );

        vello::peniko::color::DynamicColor::from_alpha_color(ac)
    }
}

// CompositeMode
impl From<BlendMode> for vello::peniko::BlendMode {
    fn from(mode: BlendMode) -> Self {
        match mode {
            BlendMode::SourceIn => VelloCompose::SrcIn.into(),
            BlendMode::SourceOut => VelloCompose::SrcOut.into(),
            BlendMode::SourceOver => VelloCompose::SrcOver.into(),
            BlendMode::DestinationOver => VelloCompose::DestOver.into(),
            BlendMode::DestinationIn => VelloCompose::DestIn.into(),
            BlendMode::DestinationOut => VelloCompose::DestOut.into(),
            BlendMode::DestinationAtop => VelloCompose::DestAtop.into(),
            BlendMode::Xor => VelloCompose::Xor.into(),
            BlendMode::SourceAtop => VelloCompose::SrcAtop.into(),
            BlendMode::Lighter => VelloCompose::PlusLighter.into(),
            BlendMode::Copy => VelloCompose::Copy.into(),
            BlendMode::Multiply => VelloMix::Multiply.into(),
            BlendMode::Modulate => VelloBlendMode::new(VelloMix::Multiply, VelloCompose::SrcAtop),
        }
    }
}

// ColorStop
impl From<ColorStop> for vello::peniko::ColorStop {
    fn from(color_stop: ColorStop) -> Self {
        vello::peniko::ColorStop {
            offset: color_stop.offset,
            color: color_stop.color.into(),
        }
    }
}

// Extend
impl From<Extend> for vello::peniko::Extend {
    fn from(extend: Extend) -> Self {
        match extend {
            Extend::Pad => vello::peniko::Extend::Pad,
            Extend::Repeat => vello::peniko::Extend::Repeat,
            Extend::Reflect => vello::peniko::Extend::Reflect,
        }
    }
}

impl From<&Extend> for vello::peniko::Extend {
    fn from(extend: &Extend) -> Self {
        match extend {
            Extend::Pad => vello::peniko::Extend::Pad,
            Extend::Repeat => vello::peniko::Extend::Repeat,
            Extend::Reflect => vello::peniko::Extend::Reflect,
        }
    }
}

// GradientKind
impl From<GradientKind> for vello::peniko::GradientKind {
    fn from(kind: GradientKind) -> Self {
        match kind {
            GradientKind::Linear { start, end } => vello::peniko::GradientKind::Linear {
                start: start.into(),
                end: end.into(),
            },
            GradientKind::Radial { center, radius } => vello::peniko::GradientKind::Radial {
                start_center: center.into(),
                start_radius: 0.0,
                end_center: center.into(),
                end_radius: radius,
            },
            GradientKind::Sweep {
                center,
                start_angle,
                end_angle,
            } => vello::peniko::GradientKind::Sweep {
                center: center.into(),
                start_angle,
                end_angle,
            },
        }
    }
}

// Gradient
impl From<Gradient> for vello::peniko::Gradient {
    fn from(gradient: Gradient) -> Self {
        let stops: Vec<vello::peniko::ColorStop> = gradient.stops.into_iter().map(|stop| stop.into()).collect();
        let stops = vello::peniko::ColorStops::from(stops.as_slice());

        vello::peniko::Gradient {
            kind: gradient.kind.into(),
            stops: stops,
            extend: gradient.extend.into(),
            interpolation_cs: ColorSpaceTag::LinearSrgb,
            hue_direction: Default::default(),
        }
    }
}

// Text
#[derive(Debug, Clone)]
pub struct VelloFont(vello::peniko::Font);

impl VelloFont {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let blob = vello::peniko::Blob::new(Arc::new(bytes.to_vec()));
        let font = vello::peniko::Font::new(blob, 0);

        Self(font)
    }
}
//
// impl Drawable<VelloBackend> for FormatedText<VelloFont> {
//     fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
//         let transform: vello::kurbo::Affine = (self.transform * scene.backend.global_transform).into();
//
//         let font = &self.font.0;
//         let font_size = skrifa::instance::Size::new(self.size);
//         let text = &self.text;
//
//         let font_ref = vello_font_to_font_ref(font).expect("Failed to load font");
//         let axes = skrifa::MetadataProvider::axes(&font_ref);
//         let variations = [("wght", 100.0), ("wdth", 500.0)];
//         let var_loc = axes.location(variations.iter().copied());
//
//         let charmap = skrifa::MetadataProvider::charmap(&font_ref);
//         let metrics = skrifa::MetadataProvider::metrics(&font_ref, font_size, &var_loc);
//         let line_height = metrics.ascent - metrics.descent + metrics.leading;
//         let glyph_metrics = skrifa::MetadataProvider::glyph_metrics(&font_ref, font_size, &var_loc);
//
//         let mut pen_x = (self.x * 2.0) as f32;
//         let mut pen_y = (self.y * 2.0) as f32;
//
//         let brush_color: vello::peniko::Color = self.color.into();
//
//         let glyphs = text
//             .chars()
//             .filter_map(|ch| {
//                 if ch == '\n' {
//                     pen_y += line_height;
//                     pen_x = 0.0;
//                     return None;
//                 }
//                 let gid = charmap.map(ch).unwrap_or_default();
//                 let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
//                 let x = pen_x;
//                 pen_x += advance;
//                 Some(vello::Glyph {
//                     id: gid.to_u32(),
//                     x,
//                     y: pen_y,
//                 })
//             })
//             .collect::<Vec<_>>();
//
//         let text_width = pen_x as f64;
//         let text_height = pen_y as f64 + line_height as f64;
//
//         let transform_x = match self.alignment {
//             Alignment::Left => 0.0,
//             Alignment::Center => -text_width / 2.0,
//             Alignment::Right => -text_width,
//         };
//
//         let transform_y = match self.vertical_alignment {
//             VerticalAlignment::Top => 0.0,
//             VerticalAlignment::Middle => text_height / 2.0,
//             VerticalAlignment::Bottom => text_height,
//         };
//
//         let transform = transform.pre_translate(vello::kurbo::Vec2::new(transform_x, transform_y));
//
//         scene
//             .backend
//             .vello_scene
//             .draw_glyphs(font)
//             .font_size(self.size)
//             .transform(transform)
//             .glyph_transform(self.glyph_transform.map(|t| t.into()))
//             .normalized_coords(bytemuck::cast_slice(var_loc.coords()))
//             .brush(brush_color)
//             .hint(false)
//             .draw(vello::peniko::Fill::NonZero, glyphs.into_iter());
//     }
// }

fn vello_font_to_font_ref(font: &vello::peniko::Font) -> Option<skrifa::FontRef<'_>> {
    use skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}
//
// impl Drawable<VelloBackend> for &PrerenderedScene {
//     fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
//         let global_transform = scene.backend.global_transform;
//         let transform = self.transform * global_transform;
//
//         scene
//             .backend
//             .vello_scene
//             .append(&mut &self.scene, Some(transform.into()));
//     }
// }
