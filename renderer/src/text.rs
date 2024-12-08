use super::{affine::Affine, colors::RGBA, shapes::Point};

/// A piece of formatted text.
#[derive(Debug, Clone)]
pub struct FormatedText<T> {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub size: f32,
    pub color: RGBA,
    pub weight: f32,
    pub font: T,
    pub style: FontStyle,
    pub alignment: Alignment,
    pub vertical_alignment: VerticalAlignment,
    pub transform: Affine,
    pub glyph_transform: Option<Affine>,
}

#[derive(Debug, Clone)]
pub enum FontStyle {
    Normal,
    Italic,
    // Oblique,
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
