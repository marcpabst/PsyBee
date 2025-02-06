use super::affine::Affine;
use super::colors;
use super::styles::{BlendMode, StrokeStyle};
use crate::brushes::Brush;
use crate::colors::RGBA;
use crate::prelude::FillStyle;
use crate::shapes::Shape;
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

    // fn draw_in_layer(
    //     &mut self,
    //     mix_mode: MixMode,
    //     composite_mode: CompositeMode,
    //     clip: Shape,
    //     clip_transform: Affine,
    //     layer_transform: Option<Affine>,
    //     alpha: f32,
    //     draw_fn: Box<dyn FnOnce(&mut Self)>,
    // ) {
    //     self.start_layer(mix_mode, composite_mode, clip, clip_transform, layer_transform, alpha);
    //     draw_fn(self);
    //     self.end_layer();
    // }

    // fn draw_alpha_mask(
    //     &mut self,
    //     mask: impl FnOnce(&mut Self),
    //     item: impl FnOnce(&mut Self),
    //     clip: Shape,
    //     clip_transform: Affine,
    // ) {
    //     self.start_layer(
    //         MixMode::Normal,
    //         CompositeMode::SourceOver,
    //         clip.clone(),
    //         clip_transform,
    //         None,
    //         1.0,
    //     );
    //     // mask(self.scene_mut());
    //
    //     self.start_layer(
    //         MixMode::Multiply,
    //         CompositeMode::SourceIn,
    //         clip.clone(),
    //         clip_transform,
    //         None,
    //         1.0,
    //     );
    //
    //     // item(self.scene_mut());
    //
    //     self.end_layer();
    //     self.end_layer();
    // }

    fn draw_shape_fill(&mut self, shape: Shape, brush: Brush, transform: Option<Affine>, blend_mode: Option<BlendMode>);
    fn draw_shape_stroke(
        &mut self,
        shape: Shape,
        brush: Brush,
        style: StrokeStyle,
        transform: Option<Affine>,
        blend_mode: Option<BlendMode>,
    );
}
