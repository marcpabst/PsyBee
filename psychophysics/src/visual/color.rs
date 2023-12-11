/// A RGBA color in sRGB space. Red, green, and blue channels are represented as floating point numbers in the range [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct StandardRGBAColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

/// A L*a*b* color in CIE1976 (CIELAB) space.
#[derive(Debug, Clone, Copy)]
pub struct CIELABColor {
    l: f64,
    a: f64,
    b: f64,
}

/// A XYZ color in CIE 1931 color space. The XYZ color space is based on the CIE 1931 color matching functions. Because all colors can be represented in this space, we use it as an intermediate representation for converting between different color spaces.
#[derive(Debug, Clone, Copy)]
pub struct XYZColor {
    x: f64,
    y: f64,
    z: f64,
}

/// A color. The specific representation in memory and the color space depends on the variant. If you want to convert a color to a specific color space, use the `into` method.
/// Please be aware that the conversion between color spaces is not lossless. For example, converting from RGB to LAB and back to RGB might result in a slightly different color.
/// For example, if you specify a color in XYZ space and then pass it to a stimulus for rendering, the color will usually be converted to sRGB space (because that is the color space of the display).
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Standard RGBA color in sRGB space.
    StandardRGBA(StandardRGBAColor),
    /// LAB color in CIE1976 (CIELAB) space.
    CIELAB(CIELABColor),
    /// XYZ color in CIE 1931 color space.
    XYZ(XYZColor),
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        let rgba: StandardRGBAColor = self.into();

        wgpu::Color {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            a: rgba.a,
        }
    }
}

impl Into<glyphon::Color> for Color {
    fn into(self) -> glyphon::Color {
        let rgba: StandardRGBAColor = self.into();

        glyphon::Color::rgba(
            float_to_byte(rgba.r),
            float_to_byte(rgba.g),
            float_to_byte(rgba.b),
            float_to_byte(rgba.a),
        )
    }
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

fn clamp_unit(x: f64) -> f64 {
    clamp(x, 0.0, 1.0)
}

impl Color {
    /// Create a new color from RGB values in standard RGB space. The values are clamped to the range [0, 1].
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        // warn if values are outside the range [0, 1]
        if r < 0.0 || r > 1.0 || g < 0.0 || g > 1.0 || b < 0.0 || b > 1.0 {
            log::warn!("Color values are outside the range [0, 1] and will be clamped");
        }
        Self::StandardRGBA(StandardRGBAColor {
            r: clamp_unit(r),
            g: clamp_unit(g),
            b: clamp_unit(b),
            a: 1.0,
        })
    }

    /// The color red as defined by sRGB(100%, 0%, 0%).
    pub const RED: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    /// The color green as defined by sRGB(0%, 100%, 0%).
    pub const GREEN: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    });
    /// The color blue as defined by sRGB(0%, 0%, 100%).
    pub const BLUE: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    });
    /// The color yellow as defined by sRGB(100%, 100%, 0%).
    pub const YELLOW: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    });
    /// The white color as defined by sRGB(100%, 100%, 100%).
    pub const WHITE: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    /// The black color as defined by sRGB(0%, 0%, 0%).
    pub const BLACK: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    /// The gray color as defined by sRGB(50%, 50%, 50%).
    pub const GRAY: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    });
    /// The transparent color as defined by sRGBA(0%, 0%, 0%, 0%).
    pub const TRANSPARENT: Color = Color::StandardRGBA(StandardRGBAColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });
}

fn float_to_byte(x: f64) -> u8 {
    (x * 255.0).round() as u8
}

// convert CIELAB to XYZ
impl From<CIELABColor> for XYZColor {
    fn from(lab: CIELABColor) -> Self {
        let y = (lab.l + 16.0) / 116.0;
        let x = lab.a / 500.0 + y;
        let z = y - lab.b / 200.0;

        let x3 = x.powi(3);
        let y3 = y.powi(3);
        let z3 = z.powi(3);

        let x = if x3 > 0.008856 { x3 } else { (x - 16.0 / 116.0) / 7.787 };
        let y = if y3 > 0.008856 { y3 } else { (y - 16.0 / 116.0) / 7.787 };
        let z = if z3 > 0.008856 { z3 } else { (z - 16.0 / 116.0) / 7.787 };

        Self {
            x: x * 95.047,
            y: y * 100.0,
            z: z * 108.883,
        }
    }
}

// convert RGB to XYZ
impl From<StandardRGBAColor> for XYZColor {
    fn from(rgba: StandardRGBAColor) -> Self {
        let r = if rgba.r > 0.04045 {
            ((rgba.r + 0.055) / 1.055).powf(2.4)
        } else {
            rgba.r / 12.92
        };
        let g = if rgba.g > 0.04045 {
            ((rgba.g + 0.055) / 1.055).powf(2.4)
        } else {
            rgba.g / 12.92
        };
        let b = if rgba.b > 0.04045 {
            ((rgba.b + 0.055) / 1.055).powf(2.4)
        } else {
            rgba.b / 12.92
        };

        let r = r * 100.0;
        let g = g * 100.0;
        let b = b * 100.0;

        Self {
            x: r * 0.4124 + g * 0.3576 + b * 0.1805,
            y: r * 0.2126 + g * 0.7152 + b * 0.0722,
            z: r * 0.0193 + g * 0.1192 + b * 0.9505,
        }
    }
}

// convert XYZ to CIELAB
impl Into<CIELABColor> for XYZColor {
    fn into(self) -> CIELABColor {
        let x = self.x / 95.047;
        let y = self.y / 100.0;
        let z = self.z / 108.883;

        let x = if x > 0.008856 {
            x.powf(1.0 / 3.0)
        } else {
            7.787 * x + 16.0 / 116.0
        };
        let y = if y > 0.008856 {
            y.powf(1.0 / 3.0)
        } else {
            7.787 * y + 16.0 / 116.0
        };
        let z = if z > 0.008856 {
            z.powf(1.0 / 3.0)
        } else {
            7.787 * z + 16.0 / 116.0
        };

        CIELABColor {
            l: 116.0 * y - 16.0,
            a: 500.0 * (x - y),
            b: 200.0 * (y - z),
        }
    }
}

// convert XYZ to RGB
impl Into<StandardRGBAColor> for XYZColor {
    fn into(self) -> StandardRGBAColor {
        let x = self.x / 100.0;
        let y = self.y / 100.0;
        let z = self.z / 100.0;

        let r = x * 3.2406 + y * -1.5372 + z * -0.4986;
        let g = x * -0.9689 + y * 1.8758 + z * 0.0415;
        let b = x * 0.0557 + y * -0.2040 + z * 1.0570;

        let r = if r > 0.0031308 {
            1.055 * r.powf(1.0 / 2.4) - 0.055
        } else {
            12.92 * r
        };
        let g = if g > 0.0031308 {
            1.055 * g.powf(1.0 / 2.4) - 0.055
        } else {
            12.92 * g
        };
        let b = if b > 0.0031308 {
            1.055 * b.powf(1.0 / 2.4) - 0.055
        } else {
            12.92 * b
        };

        StandardRGBAColor {
            r: r,
            g: g,
            b: b,
            a: 1.0,
        }
    }
}

// convert Color to raw RGBA
impl From<Color> for StandardRGBAColor {
    fn from(color: Color) -> Self {
        match color {
            Color::StandardRGBA(rgba) => rgba,
            _ => XYZColor::from(color).into(),
        }
    }
}

// convert Color to raw CIELAB
impl From<Color> for CIELABColor {
    fn from(color: Color) -> Self {
        match color {
            Color::CIELAB(lab) => lab,
            _ => XYZColor::from(color).into(),
        }
    }
}

// convert Color to raw XYZ
impl From<Color> for XYZColor {
    fn from(color: Color) -> Self {
        match color {
            Color::XYZ(xyz) => xyz,
            _ => color.into(),
        }
    }
}
