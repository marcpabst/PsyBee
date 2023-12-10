/// A color can be either RGB or RGBA with values between 0.0 and 1.0
/// Unless stated otherwise, all colors are assumed to be in sRGB color space
#[derive(Debug, Copy, Clone)]
pub enum Color {
    /// R(ed) G(reen) B(lue) color. (0.0, 0.0, 0.0) is black, (1.0, 1.0, 1.0) is white
    RGB { r: f64, g: f64, b: f64 },
    /// R(ed) G(reen) B(lue) A(lpha) color. Identical to RGB, but with an additional alpha channel
    RGBA { r: f64, g: f64, b: f64, a: f64 },
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::RGB {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        match self {
            Color::RGB { r, g, b } => wgpu::Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
                a: 1.0,
            },
            Color::RGBA { r, g, b, a } => wgpu::Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
                a: a as f64,
            },
        }
    }
}

impl Into<glyphon::Color> for Color {
    fn into(self) -> glyphon::Color {
        match self {
            Color::RGB { r, g, b } => {
                glyphon::Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
            }
            Color::RGBA { r, g, b, a } => glyphon::Color::rgba(
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            ),
        }
    }
}

impl Color {
    /// Create a new color from RGB values.
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::RGB { r, g, b }
    }
    /// Create a new color from RGBA values.
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self::RGBA { r, g, b, a }
    }
    pub const RED: Self = Self::RGB {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const GREEN: Self = Self::RGB {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const BLUE: Self = Self::RGB {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
    pub const WHITE: Self = Self::RGB {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const BLACK: Self = Self::RGB {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const YELLOW: Self = Self::RGB {
        r: 1.0,
        g: 1.0,
        b: 0.0,
    };
    pub const MAGENTA: Self = Self::RGB {
        r: 1.0,
        g: 0.0,
        b: 1.0,
    };
    pub const CYAN: Self = Self::RGB {
        r: 0.0,
        g: 1.0,
        b: 1.0,
    };
    pub const TRANSPARENT: Self = Self::RGBA {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
}
