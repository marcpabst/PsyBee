use super::affine::Affine;
use super::colors;
use super::styles::{BlendMode, ImageFitMode, StrokeStyle};
use crate::bitmaps::DynamicBitmap;
use crate::brushes::{Brush, Extend};
use crate::colors::RGBA;
use crate::prelude::FillStyle;
use crate::shapes::{Point, Shape};
use std::any::Any;

pub struct DynamicScene(pub Box<dyn Scene>);

impl DynamicScene {
    pub fn set_background_color(&mut self, color: RGBA) {
        self.0.set_background_color(color);
    }

    pub fn set_width(&mut self, width: u32) {
        self.0.set_width(width);
    }

    pub fn set_height(&mut self, height: u32) {
        self.0.set_height(height);
    }

    pub fn background_color(&self) -> RGBA {
        self.0.background_color()
    }

    pub fn width(&self) -> u32 {
        self.0.width()
    }

    pub fn height(&self) -> u32 {
        self.0.height()
    }

    pub fn start_layer(
        &mut self,
        composite_mode: BlendMode,
        clip: Shape,
        clip_transform: Option<Affine>,
        layer_transform: Option<Affine>,
        alpha: f32,
    ) {
        self.0
            .start_layer(composite_mode, clip, clip_transform, layer_transform, alpha);
    }

    pub fn end_layer(&mut self) {
        self.0.end_layer();
    }

    pub fn draw_shape_fill(
        &mut self,
        shape: Shape,
        brush: Brush,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        self.0.draw_shape_fill(shape, brush, transform, blend_mode);
    }

    pub fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    ) {
        self.0.draw_shape_stroke(shape, brush, style, transform, blend_mode);
    }

    pub fn try_as<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.0.as_any().downcast_ref::<T>()
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
        
        self.draw_shape_fill(Shape::rectangle(position, width as f64, height as f64), brush, None, blend_mode);
    }
    // fn draw_glyphs(
    //     &mut self,
    //     glyphs: &dyn Iterator<Item = Glyph>,
    //     brush: Brush,
    //     transform: Option<Affine>,
    //     blend_mode: Option<BlendMode>,
    // );
}
