use super::{affine::Affine, colors::RGBA};
use crate::shapes::Point;
use cosmic_text::Font as CosmicFont;
use custom_debug::CustomDebug;
use std::any::Any;
use std::sync::Arc;

pub struct DynamicFontFace(pub Box<dyn Font>);

impl DynamicFontFace {
    pub fn try_as<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.0.as_any().downcast_ref::<T>()
    }
}

pub trait Font: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A Glyph.
#[derive(Debug, Clone)]
pub struct Glyph {
    pub id: u32,
}

/// A piece of formatted text.
#[derive(CustomDebug)]
pub struct FormatedText {
    pub position: Point,
    pub text: String,
    pub size: f32,
    pub weight: f32,
    pub style: FontStyle,
    #[debug(skip)]
    pub font: DynamicFontFace,
    pub cosmic_font: Arc<CosmicFont>,
    pub alignment: Alignment,
    pub vertical_alignment: VerticalAlignment,
}

#[derive(Debug, Clone)]
pub enum FontStyle {
    Normal,
    Italic,
    // Oblique,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum FontWidth {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

/// Alignment of the text.
#[derive(Debug, Clone)]
pub enum Alignment {
    /// Align the text to the left.
    Left,
    /// Align the text to the center.
    Center,
    /// Align the text to the right.
    Right,
}

/// Vertical alignment of the text.
#[derive(Debug, Clone)]
pub enum VerticalAlignment {
    /// Align the text to the top.
    Top,
    /// Align the text to the center.
    Middle,
    /// Align the text to the bottom.
    Bottom,
}
