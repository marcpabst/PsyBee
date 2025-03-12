use core::fmt;
use std::{any::Any, fmt::Formatter};

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

impl fmt::Debug for DynamicFontFace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicFontFace").finish()
    }
}

impl Clone for DynamicFontFace {
    fn clone(&self) -> Self {
        DynamicFontFace(self.0.cloned())
    }
}

unsafe impl Send for DynamicFontFace {}

pub trait Typeface: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn cloned(&self) -> Box<dyn Typeface>;
}

/// A Glyph.
#[derive(Debug, Clone)]
pub struct Glyph {
    pub id: u16,
    pub position: Point,
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
