// This module defines the color types used in the psydk crate. It also
// defines some predefined colors that can be used in experiments.
//
// Accurate handling of colour in psydk experiments is a complex
// topic. This module provides a number of tools to help you with this.
//
// # Background
//
// You might be surprised to learn that there is no such thing as a simple
// RGB color in `psydk`. The reason for this is that many different
// colour spaces exist that can be represented as RGB. And since there is
// simply no way to know what RGB values mean without knowing the colour space,
// `psydk` requires you to specify the colour space when creating a
// color. This removes ambiguity and ensures that you always know what you
// are doing.
//
// ## Color spaces
//
// A color space is a mathematical model that describes how colors can be
// represented as tuples of numbers. Since human vision is trichromatic (i.e.
// we have three types of color receptors in our eyes), most color spaces
// are tristimulus color spaces. This means that all colors can be represented
// as a combination of three numbers.
//
// ### CIE XYZ colour space
//
// ...
//
// ### RGB colour spaces
//
// Many of the color spaces commonly used today in computer graphics
// are RGB color spaces. This means that the three numbers represent the amount
// of red, green, and blue light that is required to create a color. RGB colour
// spaces are defined by the three primary colors (red, green, and blue) and
// the white point. The white point is the color of a perfectly white surface
// under a given illumination, i.e. when all three primary colors are present
// in equal amounts. The most commonly used white point is the CIE standard
// illuminant D65, which is characterized by a color temperature of
// approximately 6500 K (wich apparently is the colour of daylight in
// western/central Europe).
//
// ### The electro-optical transfer function (EOTF)
//
// The electro-optical transfer function (EOTF) is a function that describes
// how the RGB values are converted to light intensity. In other words, it
// describes how the RGB values are converted to actual number of photons
// emitted by the display. In the simplest case, the EOTF is a linear function
// that simply scales the RGB values by a constant factor. However, most
// displays use a non-linear EOTF. This is mostly for historical reasons as
// CRT displays used a non-linear EOTF to compensate for the non-linear
// response of the phosphors used in the monitors. But as it turns out, the
// most commonly used EOTF today also happens to match the non-linear response
// of the human visual system reasonably well, allowing for more efficient
// encoding of the color values (more values are assigned to darker colors,
// which is where the human visual system is most sensitive). The most commonly
// used EOTF today is the sRGB transfer function, which is defined as follows:
//
// ```text
// if c <= 0.0031308
//    c * 12.92
// else
//   1.055 * c^(1.0 / 2.4) - 0.055
// ```
// where `c` is the RGB in the range [0.0, 1.0].
//
//
// # Colours in `psydk`
//
// ## Color types
//
// All colour handling in `psydk` is based on the `palette` crate
// (and this is also where new color spaces should be added). The `palette`
// crate is a very powerful crate for handling colors and color spaces. It
// provides a number of color spaces and conversion functions between them.
//
// `psydk` defines a number of color types that are based on the
// `palette` crate. These types are:
//  * `SRGBA`: An RGBA color in the sRGB color space with 32 bits of floating
//   point precision per channel.
// * `LinearSRGBA`: An RGBA color in the linear sRGB color space with 32 bits
//  of floating point precision per channel.
// * `XYZA`: An XYZA color with 32 bits of floating point precision per channel.
// * `YxyA`: A YxyA color with 32 bits of floating point precision per channel.
// * `DisplayP3RGB`: An RGBA color in the Display P3 color space with 32 bits
// of floating point precision per channel.
//
// ## Specifying colours
//
// As discussed above, `psydk` requires you to specify the color space
// when creating a color. This is done by using the appropriate color type.
// For example, if you would like to create a 50% gray color in sRGB, you can
// do so as follows:
// ```
// let grey = SRGBA::new(0.5, 0.5, 0.5, 1.0);
// ```
//
// Note that this is not (!) the same as the following:
//
// ```
// let grey = LinearSRGBA::new(0.5, 0.5, 0.5, 1.0);
// ```
//
// The first example creates a 50% gray color in sRGB color space. The
// second example creates a 50% gray color in linear sRGB color space.
// The difference between these two color spaces is that the sRGB color space
// uses a non-linear transfer function to encode the color values. In fact,
// 50% gray in the (non-linear) sRGB color space is the same as approximately
// 21% gray in the linear sRGB color space.
//
// ## Converting between color spaces
//
// The `palette` crate provides the `IntoColor` trait that allows you to
// convert between color spaces. For example, if you would like to convert
// the 50% gray color in sRGB to the linear sRGB color space, you can do so
// as follows:
// ```
// let grey = SRGBA::new(0.5, 0.5, 0.5, 1.0);
// let grey_linear = grey.into_color::<LinearSRGBA>();
// ```
//
// **Note:** Converting between color spaces is not lossless. This means that
// converting a color from one color space to another and back will not
// necessarily give you the same color.

use bytemuck::{Pod, Zeroable};
use palette::{IntoColor, Srgba, Xyza};
use wgpu::TextureFormat;

/// Macro that creates an sRGB color from a given hex value.
///
/// # Examples
/// ```
/// let red: SRGBA = srgb_hex!(0xff0000);
/// let green: SRGBA = srgb_hex!(0x00ff00);
/// let blue: SRGBA = srgb_hex!(0x0000ff);
/// ```
#[macro_export]
macro_rules! srgb_hex {
    ($hex:expr) => {{
        use $crate::visual::color::SRGBA;
        SRGBA::new(
            (($hex >> 16) & 0xff) as f32 / 255.0,
            (($hex >> 8) & 0xff) as f32 / 255.0,
            ($hex & 0xff) as f32 / 255.0,
            1.0,
        )
    }};
}

/// Represents an RGBA color value in the sRGB color space with each channel
/// having 32 bits of floating point precision.
///
/// The expected value range for each channel is [0.0, 1.0], with values encoded
/// using the sRGB transfer function.
///
/// sRGB, the standard color space for numerous consumer applications, was
/// developed by HP and Microsoft in 1996. It later gained formal recognition by
/// the International Electrotechnical Commission (IEC) under the standard IEC
/// 61966-2-1:1999. sRGB incorporates the Rec. 709 primaries, which originated
/// in 1990 for HDTV systems under the ITU-R Recommendation BT.709. The white
/// point in sRGB is the CIE standard illuminant D65, characterized by a color
/// temperature of approximately 6500 K[1].
///
/// The transfer function of sRGB is non-linear and piecewise linear,
/// approximating a gamma of 2.2. This encoding method was initially tailored
/// for the gamma characteristics of CRT displays. Coincidentally, it also
/// mimics the human visual system's response to light in daylight conditions,
/// making linearly spaced RGB values correspond to perceived linear light
/// intensity.
///
/// sRGB color space with an 8-bit color depth per channel is universally
/// supported, including in all major web browsers. It's important to note that,
/// owing to the widespread hardware support for the sRGB transfer function,
/// using this color format does not incur a performance penalty, while still
/// enabling accurate alpha blending.
///
/// [1]: Following the re-definition of several physical constants in 1968 by the International
/// Committee for Weights and Measures (CIPM), there was a minor shift in the
/// Planckian locus. As a result, the CIE standard illuminant D65 is not
/// precisely at 6500 K, but rather at 6504 K.
pub type SRGBA = palette::rgb::Srgba<f32>;

/// Represents an RGBA color value in the linear sRGB color space with each
/// channel having 32 bits of floating point precision. The expected value range
/// for each channel is [0.0, 1.0]. The primary distinction between this color
/// format and the `EncodedSRGBA` format lies in the use of a linear
/// transfer function in place of the sRGB transfer function. For further
/// details, refer to the documentation of `EncodedSRGBA`.
pub type LinearSRGBA = palette::rgb::LinSrgba<f32>;

/// Represents a color value in the CIE 1931 XYZ color space with an XYZA
/// format.
///
/// Each channel (X, Y, Z, A) uses 32-bit floating-point precision for high
/// accuracy.
///
/// The CIE 1931 XYZ color space, established by the International Commission on
/// Illumination (CIE) in 1931, is a linear and device-independent color model.
/// Its foundations lie in the human visual perception, as defined by the CIE
/// 1931 color matching functions. These functions were developed
/// from psychophysical experiments conducted in the 1920s. In this color space,
/// the XYZ values directly correlate with the light perceived by the human eye.
/// Specifically, the Y channel corresponds to the color's luminance, while the
/// X and Z channels relate to the perceived red and blue light components,
/// respectively.
pub type XYZA = palette::Xyza<palette::white_point::D65, f32>;

/// Represents a color in the Yxy color space with an YxyA format.
/// Each channel (Y, x, y, A) uses 32-bit floating-point precision for high
/// accuracy.
pub type YxyA = palette::Yxy<palette::white_point::D65, f32>;

/// Represents an RGBA color value in the Display P3 color space.
///
/// Colors in this format are represented using 32 bits of floating point
/// precision per channel.
///
/// Display P3 is a color space developed by Apple Inc. It used the primaries
/// defined for the DCI-P3 standard as outlined in the Digital Cinema System
/// Specification but instead of the approximate 6,500 K white point used in
/// DCI-P3, it uses the CIE standard illuminant D65 as its white point. It also
/// uses the same non-linear transfer function as sRGB instead of the gamma 2.6
/// transfer function used in DCI-P3.
///
/// As this is using the same transfer function as sRGB, there is no performance
/// penalty for using this color format while still relying on accurate (i.e.
/// linear) alpha blending.
///
/// Compared to sRGB, Display P3 has a wider gamut and can represent more colors
/// (about 25% more). Display P3 is supported on macOS and iOS devices and is
/// one of the color spaces supported by major web browsers.
pub type DisplayP3RGB = palette::Srgba<f32>;

// Some predefined colors.

/// The colour black as defined by the CSS Color Module Level 4 specification.
pub const BLACK: SRGBA = srgb_hex!(0x000000);
/// The colour silver as defined by the CSS Color Module Level 4 specification.
pub const SILVER: SRGBA = srgb_hex!(0xC0C0C0);
/// The colour gray as defined by the CSS Color Module Level 4 specification.
pub const GRAY: SRGBA = srgb_hex!(0x808080);
/// The colour white as defined by the CSS Color Module Level 4 specification.
pub const WHITE: SRGBA = srgb_hex!(0xFFFFFF);
/// The colour maroon as defined by the CSS Color Module Level 4 specification.
pub const MAROON: SRGBA = srgb_hex!(0x800000);
/// The colour red as defined by the CSS Color Module Level 4 specification.
pub const RED: SRGBA = srgb_hex!(0xFF0000);
/// The colour purple as defined by the CSS Color Module Level 4 specification.
pub const PURPLE: SRGBA = srgb_hex!(0x800080);
/// The colour fuchsia as defined by the CSS Color Module Level 4 specification.
pub const FUCHSIA: SRGBA = srgb_hex!(0xFF00FF);
/// The colour green as defined by the CSS Color Module Level 4 specification.
pub const GREEN: SRGBA = srgb_hex!(0x008000);
/// The colour lime as defined by the CSS Color Module Level 4 specification.
pub const LIME: SRGBA = srgb_hex!(0x00FF00);
/// The colour olive as defined by the CSS Color Module Level 4 specification.
pub const OLIVE: SRGBA = srgb_hex!(0x808000);
/// The colour yellow as defined by the CSS Color Module Level 4 specification.
pub const YELLOW: SRGBA = srgb_hex!(0xFFFF00);
/// The colour navy as defined by the CSS Color Module Level 4 specification.
pub const NAVY: SRGBA = srgb_hex!(0x000080);
/// The colour blue as defined by the CSS Color Module Level 4 specification.
pub const BLUE: SRGBA = srgb_hex!(0x0000FF);
/// The colour teal as defined by the CSS Color Module Level 4 specification.
pub const TEAL: SRGBA = srgb_hex!(0x008080);
/// The colour aqua as defined by the CSS Color Module Level 4 specification.
pub const AQUA: SRGBA = srgb_hex!(0x00FFFF);
/// Fully transparent colour.
pub const TRANSPARENT: SRGBA = SRGBA::new(0.0, 0.0, 0.0, 0.0);

/// A raw rgba color value that is angnostic of colour space and type.
/// Because of this, it is usually unwise to use this type directly and to
/// perform any color operations on it. Instead, use the `palette` crate
/// and convert to the desired color space using the `ColorFormat` enum.
///
/// The default type for this struct is f32, and values are expected to be
/// in the range [0.0, 1.0]. However, other types are supported and values
/// outside this range are allowed. However, handling of such values is
/// implementation dependent and they might be clamped to the range [0.0, 1.0].
///
/// There are several `From` implementations for this struct that allow
/// conversion from other color types.
///
/// Fields:
/// - `r`: The red channel of the color.
/// - `g`: The green channel of the color.
/// - `b`: The blue channel of the color.
/// - `a`: The alpha channel (opacity) of the color.
#[repr(C)]
#[pyo3::prelude::pyclass]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct Rgba {
    /// The red channel.
    pub r: f32,
    /// The green channel.
    pub g: f32,
    /// The blue channel.
    pub b: f32,
    /// The alpha channel.
    pub a: f32,
}

impl Rgba {
    /// Creates a new `RawRgba` color.
    ///
    /// # Arguments
    /// * `r` - The red channel.
    /// * `g` - The green channel.
    /// * `b` - The blue channel.
    /// * `a` - The alpha channel.
    ///
    /// # Returns
    /// * `RawRgba` - The new color.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Converts the color to bytes in the native endianness.
    pub fn to_ne_bytes(&self) -> [u8; 16] {
        let r = self.r.to_ne_bytes();
        let g = self.g.to_ne_bytes();
        let b = self.b.to_ne_bytes();
        let a = self.a.to_ne_bytes();
        [
            r[0], r[1], r[2], r[3], g[0], g[1], g[2], g[3], b[0], b[1], b[2], b[3], a[0], a[1], a[2], a[3],
        ]
    }
}

#[pyo3::prelude::pymethods]
impl Rgba {
    #[new]
    pub fn __new__(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }
}

impl IntoRawRgba for Rgba {
    fn convert_to_raw_rgba(&self, _color_format: ColorFormat) -> Rgba {
        *self
    }
}

/// The ColorFormat defines how color is handled internally in the rendering
/// pipeline. It is used to convert colors to the appropriate color space
/// before rendering.
#[derive(Copy, Clone, Debug)]
pub enum ColorFormat {
    /// Indicates that the rendering pipeline should use the sRGB color space
    /// with 8 bits per channel (32 bits per pixel). This is the default color
    /// format and is supported on virtually all hardware.
    SRGBA8,

    // /// The sRGB color space with 8 bits per channel (24 bits per pixel).
    // SRGBA10,
    /// Indicates that the rendering pipeline should use the Display P3 color
    /// space with 8 bits per channel (32 bits per pixel). This color format
    /// is supported on macOS and iOS devices and is one of the color spaces
    /// supported by major web browsers. Note that because the Display P3
    /// color space has a wider gamut than sRGB, it can represent more colors
    /// (about 25% more). However, as this format still uses the same bit depth
    /// as the `SRGBA8` format, color banding may be more apparent.
    DisplayP3U8,
    RGB16f,
}

pub trait IntoRawRgba {
    fn convert_to_raw_rgba(&self, color_format: ColorFormat) -> Rgba;
}

impl ColorFormat {
    /// Takes a `palette`` colour and converts it to a raw rgba color matching
    /// the colour space defined by this color format. This method should always
    /// be called when a colour is passed to the rendering pipeline.
    ///
    /// # Arguments
    /// * `col` - The colour to convert.
    ///
    /// # Returns
    /// * `RawRgba<f32>` - The converted colour.
    pub fn convert_to_raw_rgba(&self, col: impl IntoColor<Xyza<palette::white_point::D65, f32>>) -> Rgba {
        match self {
            ColorFormat::SRGBA8 => {
                let col: Xyza<palette::white_point::D65, f32> = col.into_color();
                let col: Srgba<f32> = col.into_color();
                Rgba {
                    r: col.red as f32,
                    g: col.green as f32,
                    b: col.blue as f32,
                    a: col.alpha as f32,
                }
            }
            ColorFormat::DisplayP3U8 => {
                todo!()
            }
            ColorFormat::RGB16f => {
                let col: Xyza<palette::white_point::D65, f32> = col.into_color();
                let col: Srgba<f32> = col.into_color();
                Rgba {
                    r: col.red as f32,
                    g: col.green as f32,
                    b: col.blue as f32,
                    a: col.alpha as f32,
                }
            }
        }
    }

    /// Returns the wgpu::TextureFormat that corresponds to this color format.
    pub fn to_wgpu_texture_format(&self) -> TextureFormat {
        match self {
            ColorFormat::SRGBA8 => TextureFormat::Bgra8UnormSrgb,
            ColorFormat::DisplayP3U8 => TextureFormat::Bgra8UnormSrgb,
            ColorFormat::RGB16f => TextureFormat::Bgra8Unorm,
        }
    }

    /// Returns the wgpu::TextureFormat for he swapchain and the view.
    ///
    /// # Returns
    /// * `TextureFormat` - The texture format for the swapchain.
    /// * `TextureFormat` - The texture format for the view.
    pub fn to_wgpu_swapchain_texture_format(&self) -> (TextureFormat, TextureFormat) {
        match self {
            ColorFormat::SRGBA8 => (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb),
            ColorFormat::DisplayP3U8 => (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb),
            ColorFormat::RGB16f => (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8Unorm),
        }
    }

    /// Returns the wgpu::PredefinedColorSpace for this color format. As this
    /// function can panic when the color format is not supported, it is only
    /// intended to be used when running in the browser.
    ///
    /// # Returns
    /// * `wgpu::PredefinedColorSpace` - The color space for this color format.
    ///
    /// # Panics
    /// Panics if the color format is not supported.
    pub fn get_wgpu_predefined_color_space(&self) -> wgpu::PredefinedColorSpace {
        match self {
            ColorFormat::SRGBA8 => wgpu::PredefinedColorSpace::Srgb,
            ColorFormat::DisplayP3U8 => wgpu::PredefinedColorSpace::DisplayP3,
            _ => panic!("Unsupported color format"),
        }
    }
}

/// Implements the From trait for RawRgba<T> to convert to a wgpu::Color.
impl From<Rgba> for wgpu::Color {
    fn from(col: Rgba) -> Self {
        wgpu::Color {
            r: col.r.into(),
            g: col.g.into(),
            b: col.b.into(),
            a: col.a.into(),
        }
    }
}

// implement ToRawRgba for SRGBA
impl IntoRawRgba for SRGBA {
    fn convert_to_raw_rgba(&self, color_format: ColorFormat) -> Rgba {
        match color_format {
            ColorFormat::SRGBA8 => {
                let col = self.clone();
                let col: Xyza<palette::white_point::D65, f32> = col.into_color();
                let col: Srgba<f32> = col.into_color();
                Rgba {
                    r: col.red as f32,
                    g: col.green as f32,
                    b: col.blue as f32,
                    a: col.alpha as f32,
                }
            }
            ColorFormat::DisplayP3U8 => {
                todo!()
            }
            ColorFormat::RGB16f => {
                let col = self.clone();
                let col: Xyza<palette::white_point::D65, f32> = col.into_color();
                let col: Srgba<f32> = col.into_color();
                Rgba {
                    r: col.red as f32,
                    g: col.green as f32,
                    b: col.blue as f32,
                    a: col.alpha as f32,
                }
            }
        }
    }
}
