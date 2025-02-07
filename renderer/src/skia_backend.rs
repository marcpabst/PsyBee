use crate::affine::Affine;
use crate::brushes::Extend;
use crate::brushes::{Brush, Gradient, GradientKind};
use crate::colors::RGBA;
use crate::prelude::{BlendMode, DynamicFontFace, StrokeStyle};
use crate::renderer::Renderer;
use crate::scenes::Scene;
use crate::shapes::Shape;
use foreign_types_shared::ForeignType;
use skia_safe::{PictureRecorder, SamplingOptions};
use std::any::Any;
use cosmic_text::fontdb::FaceInfo;
use wgpu::{Device, Queue, Texture};

use skia_safe::{
    gpu::{self, mtl, SurfaceOrigin},
    scalar, ColorType,
};

use skia_safe::gradient_shader::linear as sk_linear;
use skia_safe::gradient_shader::radial as sk_radial;
use skia_safe::gradient_shader::sweep as sk_sweep;
use skia_safe::image::Image as SkImage;
use skia_safe::images::raster_from_data as sk_raster_from_data;
use skia_safe::AlphaType as SkAlphaType;

use skia_safe::BlendMode as SkBlendMode;

use crate::bitmaps::{Bitmap, DynamicBitmap};
use crate::styles::ImageFitMode;
use skia_safe::gradient_shader::GradientShaderColors as SkGradientShaderColors;

#[derive(Debug)]
pub struct SkiaScene {
    pub picture_recorder: PictureRecorder,
    pub width: u32,
    pub height: u32,
    pub current_blend_mode: SkBlendMode,
}

pub struct SkiaRenderer {
    context: gpu::DirectContext,
    new_command_queue: metal::CommandQueue,
}

impl SkiaScene {
    pub fn new(width: u32, height: u32) -> Self {
        let mut picture_recorder = PictureRecorder::new();
        let bounds = skia_safe::Rect::from_wh(width as f32, height as f32);
        let _ = picture_recorder.begin_recording(bounds, None);

        Self {
            picture_recorder,
            width,
            height,
            current_blend_mode: SkBlendMode::SrcOver,
        }
    }

    fn draw_shape(skia_canvas: &skia_safe::Canvas, skia_paint: skia_safe::Paint, shape: Shape, affine: Option<Affine>) {
        // apply the affine transformation
        if let Some(affine) = affine {
            skia_canvas.save();
            skia_canvas.concat(&affine.into());
        }

        match shape {
            Shape::Rectangle { a, b } => {
                let rect = skia_safe::Rect::from_xywh(a.x as f32, a.y as f32, b.x as f32, b.y as f32);
                skia_canvas.draw_rect(rect, &skia_paint);
            }
            Shape::Circle { center, radius } => {
                skia_canvas.draw_circle(center, radius as f32, &skia_paint);
            }
            Shape::Line { start, end } => {
                skia_canvas.draw_line(start, end, &skia_paint);
            }
            Shape::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => {
                // create bounds for the ellipse
                let width = radius_x as f32;
                let height = radius_y as f32;

                let bounds = skia_safe::Rect::from_xywh(
                    center.x as f32 - width,
                    center.y as f32 - height,
                    width * 2.0,
                    height * 2.0,
                );

                // rotate the canvas
                skia_canvas.save();
                skia_canvas.rotate(rotation as f32, Some(center.into()));
                skia_canvas.draw_oval(bounds, &skia_paint);
                skia_canvas.restore();
            }
            Shape::RoundedRectangle { a, b, radius } => {
                let rect = skia_safe::Rect::from_xywh(a.x as f32, a.y as f32, b.x as f32, b.y as f32);
                skia_canvas.draw_round_rect(rect, radius as f32, radius as f32, &skia_paint);
            }
        }
        // restore the canvas
        if let Some(_) = affine {
            skia_canvas.restore();
        }
    }

    fn clip_shape(skia_canvas: &skia_safe::Canvas, skia_paint: skia_safe::Paint, shape: Shape, affine: Option<Affine>) {
        match shape {
            Shape::Rectangle { a, b } => {
                let rect = skia_safe::Rect::from_xywh(a.x as f32, a.y as f32, b.x as f32, b.y as f32);
                skia_canvas.clip_rect(rect, skia_safe::ClipOp::Intersect, true);
            }
            Shape::Circle { center, radius } => {
                let circle = skia_safe::path::Path::circle(center, radius as f32, skia_safe::path::Direction::CCW);
                skia_canvas.clip_path(&circle, skia_safe::ClipOp::Intersect, true);
            }
            _ => {
                todo!()
            }
        }
    }
}

impl Scene for SkiaScene {
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
        let mut canvas = self.picture_recorder.recording_canvas().unwrap();
        let mut layer_paint = skia_safe::Paint::default();
        layer_paint.set_alpha_f(alpha);
        layer_paint.set_blend_mode(composite_mode.into());
        let mut save_layer_rec = skia_safe::canvas::SaveLayerRec::default();
        let save_layer_rec = save_layer_rec.paint(&layer_paint);

        canvas.save_layer(&save_layer_rec);
        Self::clip_shape(&mut canvas, skia_safe::Paint::default(), clip, clip_transform);

        // update the current blend mode
        // self.current_blend_mode = composite_mode.into();
    }

    fn end_layer(&mut self) {
        self.picture_recorder.recording_canvas().unwrap().restore();
        self.current_blend_mode = SkBlendMode::SrcOver;
    }

    fn draw_shape_fill(
        &mut self,
        shape: Shape,
        brush: Brush,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        let mut canvas = self.picture_recorder.recording_canvas().unwrap();
        let mut paint: skia_safe::Paint = brush.into();

        paint.set_anti_alias(true);

        if let Some(blend_mode) = blend_mode {
            paint.set_blend_mode(blend_mode.into());
        }

        Self::draw_shape(&mut canvas, paint, shape, transform);
    }

    fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        let mut canvas = self.picture_recorder.recording_canvas().unwrap();
        let mut paint: skia_safe::Paint = brush.into();
        paint.set_stroke(true);
        paint.set_anti_alias(true);

        if let Some(blend_mode) = blend_mode {
            paint.set_blend_mode(blend_mode.into());
        }

        // set the stroke width
        paint.set_stroke_width(style.width as scalar);

        Self::draw_shape(&mut canvas, paint, shape, transform);
    }
}

impl Renderer for SkiaRenderer {
    fn render_to_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture: &Texture,
        width: u32,
        height: u32,
        scene: &mut dyn Scene,
    ) {
        let raw_texture_ptr = unsafe {
            texture
                .as_hal::<wgpu::hal::api::Metal, _, _>(|texture| texture.unwrap().raw_handle().as_ptr() as mtl::Handle)
        };

        let texture_info = unsafe { mtl::TextureInfo::new(raw_texture_ptr) };

        let backend_render_target = skia_safe::gpu::backend_render_targets::make_mtl(
            (texture.width() as i32, texture.height() as i32),
            &texture_info,
        );

        // create a new skia surface
        let mut surface = unsafe {
            gpu::surfaces::wrap_backend_render_target(
                &mut self.context,
                &backend_render_target,
                SurfaceOrigin::TopLeft,
                ColorType::RGBA8888,
                None,
                None,
            )
            .unwrap()
        };

        // clear the surface
        surface.canvas().clear(skia_safe::Color::BLACK);

        let mut canvas = surface.canvas();

        // try to downcast the scene to a SkiaScene
        let mut skia_scene = scene.as_any_mut().downcast_mut::<SkiaScene>().unwrap();
        let picture = skia_scene.picture_recorder.finish_recording_as_picture(None).unwrap();

        // draw the picture to the canvas
        canvas.draw_picture(&picture, None, None);

        // flush the surface
        self.context.flush_and_submit();
    }

    fn create_scene(&self, width: u32, heigth: u32) -> Box<dyn Scene> {
        Box::new(SkiaScene::new(width, heigth))
    }

    fn create_bitmap(&self, img: image::DynamicImage) -> DynamicBitmap {
        // extract the image data as an rgba buffer
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let buffer = rgba.into_raw();

        // create a new skia image
        let image = sk_raster_from_data(
            &skia_safe::ImageInfo::new(
                (width as i32, height as i32),
                ColorType::RGBA8888,
                SkAlphaType::Unpremul,
                None,
            ),
            &unsafe { skia_safe::Data::new_bytes(&buffer.as_slice()) },
            width as usize * 4,
        )
        .unwrap();

        DynamicBitmap(Box::new(image))
    }

    fn load_font_face(&self, face_info: &FaceInfo) -> DynamicFontFace {
        // load the font face using skia
        todo!("load the font face using skia")
    }
}

impl SkiaRenderer {
    pub fn new(_width: u32, _heigth: u32, device: &Device) -> Self {
        let command_queue = unsafe {
            device
                .as_hal::<wgpu::hal::api::Metal, _, _>(|device| device.unwrap().raw_device().lock().new_command_queue())
        };

        let raw_device_ptr = unsafe {
            device.as_hal::<wgpu::hal::api::Metal, _, _>(|device| {
                device.unwrap().raw_device().lock().as_ptr() as mtl::Handle
            })
        };

        let backend = unsafe { mtl::BackendContext::new(raw_device_ptr, command_queue.as_ptr() as mtl::Handle) };

        let skia_context = gpu::direct_contexts::make_metal(&backend, None).unwrap();

        Self {
            context: skia_context,
            new_command_queue: command_queue,
        }
    }
}

impl Bitmap for SkImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// convert a color to a skia color
impl From<RGBA> for skia_safe::Color4f {
    fn from(color: RGBA) -> Self {
        skia_safe::Color4f::new(color.r, color.g, color.b, color.a)
    }
}

impl From<&RGBA> for skia_safe::Color4f {
    fn from(color: &RGBA) -> Self {
        skia_safe::Color4f::new(color.r, color.g, color.b, color.a)
    }
}

// convert a brush to a skia paint
impl From<&Brush<'_>> for skia_safe::Paint {
    fn from(brush: &Brush) -> Self {
        let mut paint = skia_safe::Paint::default();
        match brush {
            Brush::Solid(color) => {
                let skia_color: skia_safe::Color4f = color.into();
                paint.set_color4f(skia_color, &skia_safe::ColorSpace::new_srgb());
                paint
            }
            Brush::Gradient(Gradient { extend, kind, stops }) => {
                let gradient_colors: Vec<skia_safe::Color4f> = stops.iter().map(|stop| stop.color.into()).collect();
                let gradient_colors = SkGradientShaderColors::from(gradient_colors.as_slice());
                let stops: Vec<skia_safe::scalar> = stops.iter().map(|stop| stop.offset).collect();

                // gradients need to be handled through a shader
                let shader = match kind {
                    GradientKind::Linear { start, end } => sk_linear(
                        (*start, *end),
                        gradient_colors,
                        stops.as_slice(),
                        (*extend).into(),
                        None,
                        None,
                    )
                    .unwrap(),
                    GradientKind::Radial { center, radius } => sk_radial(
                        *center,
                        *radius,
                        gradient_colors,
                        stops.as_slice(),
                        (*extend).into(),
                        None,
                        None,
                    )
                    .unwrap(),
                    GradientKind::Sweep {
                        center,
                        start_angle,
                        end_angle,
                    } => sk_sweep(
                        *center,
                        gradient_colors,
                        stops.as_slice(),
                        (*extend).into(),
                        (*start_angle, *end_angle),
                        None,
                        None,
                    )
                    .unwrap(),
                };

                paint.set_shader(shader);
                paint
            }
            Brush::Image {
                image,
                start,
                fit_mode,
                edge_mode,
                sampling,
                transform,
                alpha,
            } => {
                // downcast the image to a skia image
                let skia_image = image
                    .try_as::<SkImage>()
                    .expect("You're trying to use a non-skia image with a skia renderer");

                let local_matrix = match fit_mode {
                    ImageFitMode::Original => None,
                    ImageFitMode::Exact { width, height } => {
                        let scale_x = width / skia_image.width() as f32;
                        let scale_y = height / skia_image.height() as f32;
                        Some(skia_safe::Matrix::scale((scale_x as scalar, scale_y as scalar)))
                    }
                };

                // apply the transform
                let local_matrix = match transform {
                    None => local_matrix,
                    Some(transform) => match local_matrix {
                        Some(matrix) => {
                            let mut new_matrix = matrix.clone();
                            new_matrix.post_concat(&(*transform).into());
                            Some(new_matrix)
                        }
                        None => Some((*transform).into()),
                    },
                };

                // create a shader from the image
                let shader = skia_image.to_shader(
                    Some((edge_mode.0.into(), edge_mode.1.into())),
                    SamplingOptions::default(),
                    local_matrix.as_ref(),
                );

                paint.set_shader(shader);

                // set the alpha
                if let Some(alpha) = alpha {
                    paint.set_alpha_f(*alpha);
                }

                paint
            }
            Brush::ShapePattern { shape, latice, brush } => {
                let mut paint: skia_safe::Paint = (*brush).into();

                let path = shape.into();
                let latice = &(*latice).into();

                // create a path effect
                let path_effect = skia_safe::PathEffect::path_2d(latice, &path);

                paint.set_path_effect(path_effect);
                paint
            }
        }
    }
}

// convert Point to skia point
impl From<crate::shapes::Point> for skia_safe::Point {
    fn from(point: crate::shapes::Point) -> Self {
        skia_safe::Point::new(point.x as scalar, point.y as scalar)
    }
}

// convert Affine to skia matrix
impl From<Affine> for skia_safe::Matrix {
    fn from(affine: Affine) -> Self {
        let mut matrix = skia_safe::Matrix::default();
        let scalar_array: [scalar; 6] = affine.0.map(|x| x as scalar);
        matrix.set_affine(&scalar_array);
        matrix
    }
}

// convert Extend to skia tile mode
impl From<Extend> for skia_safe::TileMode {
    fn from(extend: Extend) -> Self {
        match extend {
            Extend::Pad => skia_safe::TileMode::Clamp,
            Extend::Repeat => skia_safe::TileMode::Repeat,
            Extend::Reflect => skia_safe::TileMode::Mirror,
        }
    }
}

// convert CompositeMode to skia blend mode
impl From<BlendMode> for skia_safe::BlendMode {
    fn from(composite_mode: crate::prelude::BlendMode) -> Self {
        match composite_mode {
            BlendMode::SourceAtop => skia_safe::BlendMode::SrcATop,
            BlendMode::SourceIn => skia_safe::BlendMode::SrcIn,
            BlendMode::SourceOut => skia_safe::BlendMode::SrcOut,
            BlendMode::SourceOver => skia_safe::BlendMode::SrcOver,
            BlendMode::DestinationAtop => skia_safe::BlendMode::DstATop,
            BlendMode::DestinationIn => skia_safe::BlendMode::DstIn,
            BlendMode::DestinationOut => skia_safe::BlendMode::DstOut,
            BlendMode::DestinationOver => skia_safe::BlendMode::DstOver,
            BlendMode::Lighter => skia_safe::BlendMode::Lighten,
            BlendMode::Copy => skia_safe::BlendMode::Src,
            BlendMode::Xor => skia_safe::BlendMode::Xor,
            BlendMode::Multiply => skia_safe::BlendMode::Multiply,
            BlendMode::Modulate => skia_safe::BlendMode::Modulate,
        }
    }
}

// convert Shape to Path
impl From<&Shape> for skia_safe::Path {
    fn from(shape: &Shape) -> Self {
        let mut path = skia_safe::Path::new();
        match shape {
            Shape::Rectangle { a, b } => {
                path.add_rect(
                    skia_safe::Rect::from_xywh(a.x as scalar, a.y as scalar, b.x as scalar, b.y as scalar),
                    None,
                );
            }
            Shape::Circle { center, radius } => {
                path.add_circle(*center, *radius as scalar, None);
            }
            Shape::Line { start, end } => {
                path.move_to(*start);
                path.line_to(*end);
            }
            Shape::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => {
                path.add_oval(
                    skia_safe::Rect::from_xywh(
                        center.x as scalar - *radius_x as scalar,
                        center.y as scalar - *radius_y as scalar,
                        *radius_x as scalar * 2.0,
                        *radius_y as scalar * 2.0,
                    ),
                    None,
                );
            }
            Shape::RoundedRectangle { a, b, radius } => {
                path.add_round_rect(
                    skia_safe::Rect::from_xywh(a.x as scalar, a.y as scalar, b.x as scalar, b.y as scalar),
                    (*radius as scalar, *radius as scalar),
                    None,
                );
            }
        }
        path
    }
}

impl From<Shape> for skia_safe::Path {
    fn from(value: Shape) -> Self {
        (&value).into()
    }
}

impl From<Brush<'_>> for skia_safe::Paint {
    fn from(value: Brush) -> Self {
        (&value).into()
    }
}
