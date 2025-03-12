use std::{
    any::Any,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{
    affine::Affine,
    styles::{BlendMode, ImageFitMode, StrokeStyle},
};
use crate::{
    bitmaps::DynamicBitmap,
    brushes::{Brush, Extend},
    colors::RGBA,
    font::{DynamicFontFace, Glyph},
    shapes::{Point, Shape},
};

pub struct DynamicScene(pub Arc<Mutex<Box<dyn Scene>>>);

impl DynamicScene {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        DynamicScene(Arc::new(Mutex::new(scene)))
    }

    pub fn inner(&self) -> MutexGuard<Box<dyn Scene>> {
        self.0.lock().unwrap()
    }

    pub fn set_background_color(&mut self, color: RGBA) {
        self.inner().set_background_color(color);
    }

    pub fn set_width(&mut self, width: u32) {
        self.inner().set_width(width);
    }

    pub fn set_height(&mut self, height: u32) {
        self.inner().set_height(height);
    }

    pub fn background_color(&self) -> RGBA {
        self.inner().background_color()
    }

    pub fn width(&self) -> u32 {
        self.inner().width()
    }

    pub fn height(&self) -> u32 {
        self.inner().height()
    }

    pub fn start_layer(
        &mut self,
        composite_mode: BlendMode,
        clip: Shape,
        clip_transform: Option<Affine>,
        layer_transform: Option<Affine>,
        alpha: f32,
    ) {
        self.inner()
            .start_layer(composite_mode, clip, clip_transform, layer_transform, alpha);
    }

    pub fn end_layer(&mut self) {
        self.inner().end_layer();
    }

    pub fn draw_shape_fill(
        &mut self,
        shape: Shape,
        brush: Brush,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        self.inner().draw_shape_fill(shape, brush, transform, blend_mode);
    }

    pub fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        self.inner()
            .draw_shape_stroke(shape, brush, style, transform, blend_mode);
    }

    pub fn draw_glyphs(
        &mut self,
        position: Point,
        glyphs: &[Glyph],
        font_face: &DynamicFontFace,
        font_size: f32,
        brush: Brush,
        alpha: Option<f32>,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        self.inner().draw_glyphs(
            position, glyphs, font_face, font_size, brush, alpha, transform, blend_mode,
        );
    }
}

pub trait Scene: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn set_background_color(&mut self, color: RGBA);
    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);
    fn background_color(&self) -> RGBA;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn start_layer(
        &mut self,
        composite_mode: BlendMode,
        clip: Shape,
        clip_transform: Option<Affine>,
        layer_transform: Option<Affine>,
        alpha: f32,
    );
    fn end_layer(&mut self);
    fn draw_shape_fill(&mut self, shape: Shape, brush: Brush, transform: Option<Affine>, blend_mode: Option<BlendMode>);
    fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    );
    fn draw_image(
        &mut self,
        bitmap: &DynamicBitmap,
        position: Point,
        width: f32,
        height: f32,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
        alpha: Option<f32>,
    ) {
        let brush = Brush::Image {
            image: &bitmap,
            start: position,
            fit_mode: ImageFitMode::Exact { width, height },
            sampling: Default::default(),
            edge_mode: (Extend::Pad, Extend::Pad),
            transform,
            alpha,
        };

        self.draw_shape_fill(
            Shape::rectangle(position, width as f64, height as f64),
            brush,
            None,
            blend_mode,
        );
    }
    fn draw_glyphs(
        &mut self,
        position: Point,
        glyphs: &[Glyph],
        font_face: &DynamicFontFace,
        font_size: f32,
        brush: Brush,
        alpha: Option<f32>,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    );
}
