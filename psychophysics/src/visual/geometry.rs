use nalgebra::Matrix3;
use nalgebra::Matrix4;
use winit::window;

use super::pwindow::WindowHandle;

/// The Unit enum is used to specify the size of a stimulus. The unit can be specified in different ways,
/// which will be evaluated just before the stimulus is rendered. This allows for the size of the stimulus to
/// be specified in a flexible way, e.g. as a fraction of the screen size or in degrees of visual angle.
pub enum Unit {
    // Physical pixels
    Pixels(f64),
    /// Fraction of the screen height.
    ScreenHeight(f64),
    /// Fraction of the screen width.
    ScreenWidth(f64),
    /// Degrees of visual angle.
    Deegrees(f64),
    /// Millimeters.
    Millimeters(f64),
    /// Centimeters.
    Centimeters(f64),
    /// Inches.
    Inches(f64),
    /// Points.
    Points(f64),
    /// Defaults to the default unit set in the window (pixels if not specified otherwise).
    Default(f64),
    /// Qutioent of two units
    Quotient(Box<Unit>, Box<Unit>),
    /// Product of two units
    Product(Box<Unit>, Box<Unit>),
    /// Sum of two units
    Sum(Box<Unit>, Box<Unit>),
    /// Difference of two units
    Difference(Box<Unit>, Box<Unit>),
}

impl From<i64> for Unit {
    /// Convert from an integer to a unit. The integer is interpreted as a number of `Default` units.
    fn from(i: i64) -> Self {
        Unit::Default(i as f64)
    }
}

impl From<f64> for Unit {
    /// Convert from a float to a unit. The float is interpreted as a number of `Default` units.
    fn from(f: f64) -> Self {
        Unit::Default(f)
    }
}

impl std::ops::Add for Unit {
    type Output = Unit;
    /// Add two units together. The results is a `Unit::Sum`.
    fn add(self, rhs: Self) -> Self::Output {
        Unit::Sum(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Sub for Unit {
    type Output = Unit;
    /// Subtract two units. The results is a `Unit::Difference`.
    fn sub(self, rhs: Self) -> Self::Output {
        Unit::Difference(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Mul for Unit {
    type Output = Unit;
    /// Multiply two units. The results is a `Unit::Product`.
    fn mul(self, rhs: Self) -> Self::Output {
        Unit::Product(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Div for Unit {
    type Output = Unit;
    /// Divide two units. The results is a `Unit::Quotient`.
    fn div(self, rhs: Self) -> Self::Output {
        Unit::Quotient(Box::new(self), Box::new(rhs))
    }
}

impl Unit {
    /// Convert the given angle in degrees to a distance in millimeters.
    fn angle_to_milimeter(angle: f64, viewing_distance_mm: f64) -> Unit {
        Unit::Millimeters(2.0 * viewing_distance_mm * (angle.to_radians() / 2.0).tan())
    }

    /// Convert the given unit to pixels, taking the physical size of the screen and the viewing distance into account.
    ///
    /// # Arguments
    ///
    /// * `width_mm` - The width of the screen in millimeters.
    /// * `width_px` - The width of the screen in pixels.
    /// * `viewing_distance_mm` - The viewing distance in millimeters.
    /// * `height_px` - The height of the screen in pixels.
    ///
    /// # Returns
    ///
    /// The unit converted to pixels.
    pub fn to_pixels(&self, width_mm: f64, width_px: i32, viewing_distance_mm: f64, height_px: i32) -> f64 {
        let window_width_mm = width_mm;
        let window_width_pixels = width_px as f64;

        let window_height_mm = window_width_mm * height_px as f64 / width_px as f64;
        let window_height_pixels = height_px as f64;

        match self {
            Unit::Pixels(pixels) => *pixels,
            Unit::ScreenWidth(normalised) => *normalised * window_width_pixels,
            Unit::ScreenHeight(normalised) => *normalised * window_height_pixels,
            Unit::Deegrees(degrees) => Unit::angle_to_milimeter(*degrees, viewing_distance_mm).to_pixels(
                width_mm,
                width_px,
                viewing_distance_mm,
                height_px,
            ),
            Unit::Millimeters(millimeters) => *millimeters * window_width_pixels / window_width_mm,
            Unit::Centimeters(centimeters) => {
                Unit::Millimeters(*centimeters * 10.0).to_pixels(width_mm, width_px, viewing_distance_mm, height_px)
            }
            Unit::Inches(inches) => {
                Unit::Millimeters(*inches * 25.4).to_pixels(width_mm, width_px, viewing_distance_mm, height_px)
            }
            Unit::Points(points) => {
                Unit::Inches(*points / 72.0).to_pixels(width_mm, width_px, viewing_distance_mm, height_px)
            }
            Unit::Default(default) => *default * window_width_pixels / window_width_mm,
            Unit::Quotient(a, b) => {
                let a = a.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                let b = b.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                a / b
            }
            Unit::Product(a, b) => {
                let a = a.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                let b = b.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                a * b
            }
            Unit::Sum(a, b) => {
                let a = a.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                let b = b.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                a + b
            }
            Unit::Difference(a, b) => {
                let a = a.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                let b = b.to_pixels(window_width_mm, width_px, viewing_distance_mm, height_px);
                a - b
            }
        }
    }

    /// Convert to NDC coordinates (i.e. between -1 and 1 with the origin in the
    /// center of the screen and the point (-1, -1) in the top left corner). Internally, this function
    /// calls `to_pixels` and then converts the result to NDC coordinates.
    ///
    /// # Arguments
    ///
    /// * `width_mm` - The width of the screen in millimeters.
    /// * `width_px` - The width of the screen in pixels.
    /// * `viewing_distance_mm` - The viewing distance in millimeters.
    /// * `height_px` - The height of the screen in pixels.
    ///
    /// # Returns
    ///
    /// The unit converted to NDC coordinates.
    pub fn to_ndc(&self, width_mm: f64, width_px: i32, viewing_distance_mm: f64, height_px: i32) -> f64 {
        let pixels = self.to_pixels(width_mm, width_px, viewing_distance_mm, height_px);
        let ndc = pixels / (width_px as f64 / 2.0);
        ndc
    }
}

// implement pretty printing for units
impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Pixels(pixels) => write!(f, "{}px", pixels),
            Unit::ScreenWidth(normalised) => write!(f, "{}w", normalised),
            Unit::ScreenHeight(normalised) => write!(f, "{}h", normalised),
            Unit::Deegrees(degrees) => write!(f, "{}deg", degrees),
            Unit::Millimeters(millimeters) => write!(f, "{}mm", millimeters),
            Unit::Centimeters(centimeters) => write!(f, "{}cm", centimeters),
            Unit::Inches(inches) => write!(f, "{}in", inches),
            Unit::Points(points) => write!(f, "{}pt", points),
            Unit::Default(default) => write!(f, "{}def", default),
            Unit::Quotient(a, b) => write!(f, "({:?})/({:?})", a, b),
            Unit::Product(a, b) => write!(f, "({:?})*({:?})", a, b),
            Unit::Sum(a, b) => write!(f, "({:?})+({:?})", a, b),
            Unit::Difference(a, b) => write!(f, "({:?})-({:?})", a, b),
        }
    }
}

// We could implement shapes as an enum, but this would make it (a) hard to extend and (b) make it hard to
// define specific methods for specific shapes. Instead, we define a trait that all shapes must implement.

/// Types that can be triangulated, i.e. converted to a list of vertices.
pub trait ToVertices {
    /// Convert the shape to a list of vertices in pixels. The vertices are given as a list of floats,
    /// where each three floats represent the x, y, and z coordinate of a vertex. The z coordinate is
    /// always 0.0. X and y coordinates are given in NDC (Normalized Device Coordinates) space, i.e. between -1
    /// and 1 with the origin in the center of the screen and the point (-1, -1) in the top left corner.
    fn to_vertices_ndc(&self, width_mm: f64, viewing_distance_mm: f64, width_px: i32, height_px: i32) -> Vec<Vertex>;
}
pub struct Rectangle {
    pub left: Unit,
    pub top: Unit,
    pub width: Unit,
    pub height: Unit,
}

pub struct Circle {
    pub center_x: Unit,
    pub center_y: Unit,
    pub radius: Unit,
}

impl Rectangle {
    pub fn new(left: impl Into<Unit>, top: impl Into<Unit>, width: impl Into<Unit>, height: impl Into<Unit>) -> Self {
        Self {
            left: left.into(),
            top: top.into(),
            width: width.into(),
            height: height.into(),
        }
    }
}

impl Circle {
    pub fn new(center_x: impl Into<Unit>, center_y: impl Into<Unit>, radius: impl Into<Unit>) -> Self {
        Self {
            center_x: center_x.into(),
            center_y: center_y.into(),
            radius: radius.into(),
        }
    }
}

impl ToVertices for Rectangle {
    fn to_vertices_ndc(&self, width_mm: f64, viewing_distance_mm: f64, width_px: i32, height_px: i32) -> Vec<Vertex> {
        let left = self.left.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);
        let top = self.top.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);
        let width = self.width.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);
        let height = self.height.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);

        let vertices = vec![
            Vertex {
                position: [left, top, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [left + width, top, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [left + width, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [left, top, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [left + width, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [left, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
            },
        ];
    }
}

impl ToVertices for Circle {
    fn to_vertices_ndc(&self, width_mm: f64, viewing_distance_mm: f64, width_px: i32, height_px: i32) -> Vec<Vertex> {
        let center_x = self.center_x.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);
        let center_y = self.center_y.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);
        let radius = self.radius.to_ndc(width_mm, width_px, viewing_distance_mm, height_px);

        let mut vertices = Vec::new();

        for i in 0..360 {
            let angle = i as f64 * std::f64::consts::PI / 180.0;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            vertices.push(Vertex {
                position: [x, y, 0.0],
                color: [1.0, 1.0, 1.0],
            });
        }
    }
}

enum Transformation2D {
    /// Identity transformation (no transformation).
    Identity,
    /// Rotation around the center of the stimulus.
    RotationCenter(f32),
    /// Rotation around an arbitrary point.
    RotationPoint(f32, Unit, Unit),
    /// Scale around the center of the stimulus.
    ScaleCenter(f32, f32),
    /// Scale around an arbitrary point.
    ScalePoint(f32, f32, f32, f32),
    /// Shear around the center of the stimulus.
    ShearCenter(f32, f32),
    /// Shear around an arbitrary point.
    ShearPoint(f32, f32, f32, f32),
    /// Translation by x and y.
    Translation(Unit, Unit),
    /// Arbitrary 2D transformation matrix.
    Matrix(Matrix3<f32>),
    /// Homogeneous 2D transformation matrix. This 4x4 matrix will be applied to the coordinates in NDC (Normalized
    /// Device Coordinates) space, but please note that the specific coordinate system this matrix will be applied to is
    /// considered an implementation detail and may change in the future. It is recommended to use the other variants
    /// instead or to combine a 2D transformation matrix with a `Translation` transformation, which will take care of
    /// the coordinate system for you.
    Homogeneous(Matrix4<f32>),
    /// Product of two transformations. This variant is used to combine multiple transformations in a lazy way.
    Product(Box<Transformation2D>, Box<Transformation2D>),
}

fn rotation_matrix(angle: f32) -> Matrix4<f32> {
    let angle = angle.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();

    Matrix4::new(
        cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0., 0., 0., 1.0,
    )
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
