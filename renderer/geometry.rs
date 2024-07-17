use std::hash::Hash;
use std::hash::Hasher;

use super::helpers::CacheEntry;
use super::helpers::Fingerprint;
use super::material::Colour;
use super::material::Material;

/// A geometry object defined by what to render (a primitive and tessellation) and how to render it (a material).
pub struct Geom {
    /// Internal id for caching.
    #[allow(dead_code)]
    id: CacheEntry,
    /// The primitive to render.
    pub primitive: Primitive,
    /// The material used to render the primitive.
    pub material: Material,
    /// The transform to apply during rendering.
    pub transform: Option<Transformation>,
    /// The pixel filters to apply during rendering.
    pub filters: Vec<PixelFilter>,
    /// How to tesselate the primitive before rendering.
    pub options: TessellationOptions,
}

impl Geom {
    /// Create a new geometry object.
    pub fn new(
        primitive: Primitive,
        material: Material,
        transform: Option<Transformation>,
        filters: Vec<PixelFilter>,
        options: TessellationOptions,
    ) -> Self {
        Geom {
            id: CacheEntry::new(),
            primitive,
            material,
            transform,
            filters,
            options,
        }
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
#[repr(C)]
pub struct BBox {
    pub aa: Point2D,
    pub bb: Point2D,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Primitive {
    /// A Rectangle defined by two points.
    Rectangle { a: Point2D, b: Point2D },
    /// A rounded rectangle defined by two points and a radius.
    RoundedRectangle { a: Point2D, b: Point2D, radius: f32 },
    /// A Circle defined by a center point and a radius.
    Circle { center: Point2D, radius: f32 },
    /// A Triangle defined by three points.
    Triangle { a: Point2D, b: Point2D, c: Point2D },
    /// A polygon defined by a list of points.
    Polygon { points: Vec<Point2D> },
    /// A line defined by two points.
    Line { a: Point2D, b: Point2D },
    /// A path defined by a list of points.
    Path { points: Vec<Point2D> },
    /// An ellipse defined by a center point and two radii.
    Ellipse { center: Point2D, radii: Vector2 },
}

// make Primitive Eq
impl std::cmp::Eq for Primitive {}

impl Fingerprint for Primitive {
    fn fingerprint(&self) -> u64 {
        let mut state = std::collections::hash_map::DefaultHasher::new();
        match self {
            Primitive::Rectangle { a, b } => {
                a.x.to_bits().hash(&mut state);
                a.y.to_bits().hash(&mut state);
                b.x.to_bits().hash(&mut state);
                b.y.to_bits().hash(&mut state);
            }
            Primitive::RoundedRectangle { a, b, radius } => {
                a.x.to_bits().hash(&mut state);
                a.y.to_bits().hash(&mut state);
                b.x.to_bits().hash(&mut state);
                b.y.to_bits().hash(&mut state);
                radius.to_bits().hash(&mut state);
            }
            Primitive::Circle { center, radius } => {
                center.x.to_bits().hash(&mut state);
                center.y.to_bits().hash(&mut state);
                radius.to_bits().hash(&mut state);
            }
            Primitive::Triangle { a, b, c } => {
                a.x.to_bits().hash(&mut state);
                a.y.to_bits().hash(&mut state);
                b.x.to_bits().hash(&mut state);
                b.y.to_bits().hash(&mut state);
                c.x.to_bits().hash(&mut state);
                c.y.to_bits().hash(&mut state);
            }
            Primitive::Polygon { points } => {
                for point in points {
                    point.x.to_bits().hash(&mut state);
                    point.y.to_bits().hash(&mut state);
                }
            }
            Primitive::Line { a, b } => {
                a.x.to_bits().hash(&mut state);
                a.y.to_bits().hash(&mut state);
                b.x.to_bits().hash(&mut state);
                b.y.to_bits().hash(&mut state);
            }
            Primitive::Ellipse { center, radii } => {
                center.x.to_bits().hash(&mut state);
                center.y.to_bits().hash(&mut state);
                radii.x.to_bits().hash(&mut state);
                radii.y.to_bits().hash(&mut state);
            }
            _ => todo!(),
        }

        // add the variant to the hash
        std::mem::discriminant(self).hash(&mut state);

        state.finish()
    }
}

impl Primitive {
    /// Get bounding box of the primitive.
    pub fn bbox(&self) -> BBox {
        match self {
            Primitive::Rectangle { a, b } => {
                let aa = Point2D {
                    x: a.x.min(b.x),
                    y: a.y.min(b.y),
                };
                let bb = Point2D {
                    x: a.x.max(b.x),
                    y: a.y.max(b.y),
                };
                BBox { aa, bb }
            }
            Primitive::RoundedRectangle { a, b, .. } => {
                let aa = Point2D {
                    x: a.x.min(b.x),
                    y: a.y.min(b.y),
                };
                let bb = Point2D {
                    x: a.x.max(b.x),
                    y: a.y.max(b.y),
                };
                BBox { aa, bb }
            }
            Primitive::Circle { center, radius } => {
                let aa = Point2D {
                    x: center.x - radius,
                    y: center.y - radius,
                };
                let bb = Point2D {
                    x: center.x + radius,
                    y: center.y + radius,
                };
                BBox { aa, bb }
            }
            Primitive::Triangle { a, b, c } => {
                let aa = Point2D {
                    x: a.x.min(b.x).min(c.x),
                    y: a.y.min(b.y).min(c.y),
                };
                let bb = Point2D {
                    x: a.x.max(b.x).max(c.x),
                    y: a.y.max(b.y).max(c.y),
                };
                BBox { aa, bb }
            }
            Primitive::Polygon { points } => {
                let mut aa = Point2D {
                    x: f32::INFINITY,
                    y: f32::INFINITY,
                };
                let mut bb = Point2D {
                    x: f32::NEG_INFINITY,
                    y: f32::NEG_INFINITY,
                };
                for point in points {
                    aa.x = aa.x.min(point.x);
                    aa.y = aa.y.min(point.y);
                    bb.x = bb.x.max(point.x);
                    bb.y = bb.y.max(point.y);
                }
                BBox { aa, bb }
            }
            Primitive::Line { a, b } => {
                let aa = Point2D {
                    x: a.x.min(b.x),
                    y: a.y.min(b.y),
                };
                let bb = Point2D {
                    x: a.x.max(b.x),
                    y: a.y.max(b.y),
                };
                BBox { aa, bb }
            }
            Primitive::Ellipse { center, radii } => {
                // calculate the bounding box of the ellipse
                let aa = Point2D {
                    x: center.x - radii.x,
                    y: center.y - radii.y,
                };
                let bb = Point2D {
                    x: center.x + radii.x,
                    y: center.y + radii.y,
                };
                BBox { aa, bb }
            }
            _ => todo!(),
        }
    }
}

impl Fingerprint for &Primitive {
    fn fingerprint(&self) -> u64 {
        (*self).fingerprint()
    }
}

/// The type of line cap.
pub enum LineCap {
    Butt,
    Square,
    Round,
}

/// The type of line join.
pub enum LineJoin {
    Miter,
    MiterClip,
    Round,
    Bevel,
}

pub enum TessellationOptions {
    Fill,
    Stroke {
        start_cap: LineCap,
        end_cap: LineCap,
        line_join: LineJoin,
        line_width: f32,
        miter_limit: f32,
    },
}

impl Fingerprint for TessellationOptions {
    fn fingerprint(&self) -> u64 {
        let mut state = std::collections::hash_map::DefaultHasher::new();
        match self {
            TessellationOptions::Fill => {
                "fill".hash(&mut state);
            }
            TessellationOptions::Stroke {
                start_cap,
                end_cap,
                line_join,
                line_width,
                miter_limit,
            } => {
                match start_cap {
                    LineCap::Butt => "butt".hash(&mut state),
                    LineCap::Square => "square".hash(&mut state),
                    LineCap::Round => "round".hash(&mut state),
                }
                match end_cap {
                    LineCap::Butt => "butt".hash(&mut state),
                    LineCap::Square => "square".hash(&mut state),
                    LineCap::Round => "round".hash(&mut state),
                }
                match line_join {
                    LineJoin::Miter => "miter".hash(&mut state),
                    LineJoin::MiterClip => "miter_clip".hash(&mut state),
                    LineJoin::Round => "round".hash(&mut state),
                    LineJoin::Bevel => "bevel".hash(&mut state),
                }
                line_width.to_bits().hash(&mut state);
                miter_limit.to_bits().hash(&mut state);
            }
        }

        // add the variant to the hash
        std::mem::discriminant(self).hash(&mut state);

        state.finish()
    }
}

impl Fingerprint for &TessellationOptions {
    fn fingerprint(&self) -> u64 {
        (*self).fingerprint()
    }
}

impl TessellationOptions {
    pub fn simple_line(width: f32) -> Self {
        TessellationOptions::Stroke {
            start_cap: LineCap::Butt,
            end_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            line_width: width,
            miter_limit: 4.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// A 3x3 transformation matrix.
pub struct Transformation {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    _pad1: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
    _pad2: f32,
    pub g: f32,
    pub h: f32,
    pub i: f32,
    _pad3: [f32; 5],
}

impl Transformation {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            _pad1: 0.0,
            d: 0.0,
            e: 1.0,
            f: 0.0,
            _pad2: 0.0,
            g: 0.0,
            h: 0.0,
            i: 1.0,
            _pad3: [0.0; 5],
        }
    }
}

impl From<nalgebra::Matrix3<f32>> for Transformation {
    fn from(matrix: nalgebra::Matrix3<f32>) -> Self {
        Self {
            a: matrix[(0, 0)],
            b: matrix[(0, 1)],
            c: matrix[(0, 2)],
            _pad1: 0.0,
            d: matrix[(1, 0)],
            e: matrix[(1, 1)],
            f: matrix[(1, 2)],
            _pad2: 0.0,
            g: matrix[(2, 0)],
            h: matrix[(2, 1)],
            i: matrix[(2, 2)],
            _pad3: [0.0; 5],
        }
    }
}

/// A filter that is applied at the pixel level.
pub enum PixelFilter {
    /// A simple grayscale filter that averages the RGB values.
    Grayscale,
    /// Invert filter that inverts the RGB values.
    Invert,
    /// A per-channel threshold filter.
    Threshold { threshold: Colour },
    /// A Gaussian envelope filter.
    GaussianEnvelope {
        /// The center of the envelope, relative to geometry.
        center: Point2D,
        /// The standard deviation of the envelope in x and y.
        sigma: Vector2,
        /// The rotation of the envelope in degrees.
        rotation: f32,
    },
}

/// A hashable f32 that de-references to a f32.
#[derive(Debug, Clone, Copy)]
pub struct HashableF32(f32);

impl std::hash::Hash for HashableF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl std::ops::Deref for HashableF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
