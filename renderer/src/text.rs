use std::{any::Any, sync::Arc};

use custom_debug::CustomDebug;

use crate::shapes::Point;

pub struct DynamicFontFace(pub Box<dyn Typeface>);

impl DynamicFontFace {
    pub fn try_as<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.0.as_any().downcast_ref::<T>()
    }
}

pub trait Typeface: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A Glyph.
#[derive(Debug, Clone)]
pub struct Glyph {
    pub id: u16,
    pub position: Point,
    pub start: u32,
    pub end: u32,
}

/// A piece of formatted text.
#[derive(CustomDebug)]
pub struct FormatedText {
    pub position: Point,
    pub cosmic_buffer: cosmic_text::Buffer,
    pub comic_metrics: cosmic_text::Metrics,
    pub cosmic_font: Arc<cosmic_text::Font>,
    #[debug(skip)]
    pub renderer_font: DynamicFontFace,
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
