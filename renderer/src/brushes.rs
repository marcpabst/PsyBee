use crate::{affine::Affine, bitmaps::DynamicBitmap, colors::RGBA, shapes::Point, styles::ImageFitMode};

#[derive(Debug, Clone)]
pub enum Brush<'a> {
    /// Solid color brush.
    Solid(RGBA),
    /// Gradient brush.
    Gradient(Gradient),
    /// GPU texture brush.
    Image {
        /// The image to use as a brush.
        image: &'a DynamicBitmap,
        /// The starting point of the image. TODO: maybe rename to offset or translate?
        start: Point,
        /// The fit mode of the image.
        fit_mode: ImageFitMode,
        /// The sampling mode of the image.
        sampling: ImageSampling,
        /// The edge mode of the image.
        edge_mode: (Extend, Extend),
        /// Optional affine transform to apply to the image. This will be applied after the
        /// fit mode and start point.
        transform: Option<Affine>,
        /// Optional alpha value to apply to the image.
        alpha: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageColor {
    /// linear RGB color
    LinearRGB,
    /// sRGB color
    SRGB,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ImageSampling {
    /// Nearest neighbor sampling.
    #[default]
    Nearest,
    /// Linear sampling.
    Linear,
}

pub trait ImageData {}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub extend: Extend,
    pub kind: GradientKind,
    pub stops: Vec<ColorStop>,
}

impl Gradient {
    pub fn new_equidistant(extend: Extend, kind: GradientKind, colors: &[RGBA]) -> Self {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, color)| ColorStop {
                offset: i as f32 / (colors.len() - 1) as f32,
                color: *color,
            })
            .collect();

        Self { extend, kind, stops }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Extend {
    /// Extends the image by repeating the edge color of the brush.
    Pad,
    /// Extends the image by repeating the brush.
    Repeat,
    /// Extends the image by reflecting the brush.
    Reflect,
}

#[derive(Debug, Clone)]
pub struct ColorStop {
    /// Normalized offset of the stop.
    pub offset: f32,
    /// Color at the specified offset.
    pub color: RGBA,
}

#[derive(Debug, Clone)]
pub enum GradientKind {
    /// Gradient that transitions between two or more colors along a line.
    Linear {
        /// Starting point.
        start: Point,
        /// Ending point.
        end: Point,
    },
    /// Gradient that transitions between two or more colors that radiate from an origin.
    Radial {
        /// Center of circle.
        center: Point,
        /// Radius of circle.
        radius: f32,
    },
    /// Gradient that transitions between two or more colors that rotate around a center
    /// point.
    Sweep {
        /// Center point.
        center: Point,
        /// Start angle of the sweep, counter-clockwise of the x-axis.
        start_angle: f32,
        /// End angle of the sweep, counter-clockwise of the x-axis.
        end_angle: f32,
    },
}

// allow Extend to be converted into (Extend, Extend)
impl From<Extend> for (Extend, Extend) {
    fn from(extend: Extend) -> Self {
        (extend, extend)
    }
}
