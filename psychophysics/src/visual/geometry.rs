use nalgebra::{Matrix3, Matrix4};
use num_traits::Num;

/// The Unit enum is used to specify the size of a stimulus. The unit can be specified in different ways,
/// which will be evaluated just before the stimulus is rendered. This allows for the size of the stimulus to
/// be specified in a flexible way, e.g. as a fraction of the screen size or in degrees of visual angle.
pub enum Size {
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
    /// Qutioent of a unit and a float (the Unit divided by the float).
    Quotient(Box<Size>, f64),
    /// Product of a unit and a float (the Unit multiplied by the float).
    Product(Box<Size>, f64),
    /// Sum of two units
    Sum(Box<Size>, Box<Size>),
    /// Difference of two units
    Difference(Box<Size>, Box<Size>),
}

impl From<i64> for Size {
    /// Convert from an integer to a unit. The integer is interpreted as a number of `Default` units.
    fn from(i: i64) -> Self {
        Size::Default(i as f64)
    }
}

impl From<f64> for Size {
    /// Convert from a float to a unit. The float is interpreted as a number of `Default` units.
    fn from(f: f64) -> Self {
        Size::Default(f)
    }
}

impl std::ops::Add for Size {
    type Output = Size;
    /// Add two units together. The results is a `Unit::Sum`.
    fn add(self, rhs: Self) -> Self::Output {
        Size::Sum(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Sub for Size {
    type Output = Size;
    /// Subtract two units. The results is a `Unit::Difference`.
    fn sub(self, rhs: Self) -> Self::Output {
        Size::Difference(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Mul<f64> for Size {
    type Output = Size;
    /// Multiply two units. The results is a `Unit::Product`.
    fn mul(self, rhs: f64) -> Self::Output {
        Size::Product(Box::new(self), rhs)
    }
}

impl std::ops::Div<f64> for Size {
    type Output = Size;
    /// Divide two units. The results is a `Unit::Quotient`.
    fn div(self, rhs: f64) -> Self::Output {
        Self::Quotient(Box::new(self), rhs)
    }
}

impl Size {
    /// Convert the given angle in degrees to a distance in millimeters.
    fn angle_to_milimeter(angle: f64, viewing_distance_mm: f64) -> Size {
        Size::Millimeters(
            2.0 * viewing_distance_mm * (angle.to_radians() / 2.0).tan(),
        )
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
    pub fn to_pixels(
        &self,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> f64 {
        let window_width_mm = width_mm;
        let window_width_pixels = width_px as f64;

        //let window_height_mm = window_width_mm * height_px as f64 / width_px as f64;
        let window_height_pixels = height_px as f64;

        match self {
            Size::Pixels(pixels) => *pixels,
            Size::ScreenWidth(normalised) => *normalised * window_width_pixels,
            Size::ScreenHeight(normalised) => {
                *normalised * window_height_pixels
            }
            Size::Deegrees(degrees) => {
                Size::angle_to_milimeter(*degrees, viewing_distance_mm)
                    .to_pixels(
                        width_mm,
                        viewing_distance_mm,
                        width_px,
                        height_px,
                    )
            }
            Size::Millimeters(millimeters) => {
                *millimeters * window_width_pixels / window_width_mm
            }
            Size::Centimeters(centimeters) => {
                Size::Millimeters(*centimeters * 10.0).to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                )
            }
            Size::Inches(inches) => Size::Millimeters(*inches * 25.4)
                .to_pixels(width_mm, viewing_distance_mm, width_px, height_px),
            Size::Points(points) => Size::Inches(*points / 72.0).to_pixels(
                width_mm,
                viewing_distance_mm,
                width_px,
                height_px,
            ),
            Size::Default(default) => *default,
            Size::Quotient(a, b) => {
                // first, we resolve `a` to pixels, the we divide by b
                let a = a.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                a / b
            }
            Size::Product(a, b) => {
                // first, we resolve `a` to pixels, the we multiply with b
                let a = a.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                a * b
            }
            Size::Sum(a, b) => {
                let a = a.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                let b = b.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                a + b
            }
            Size::Difference(a, b) => {
                let a = a.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                let b = b.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                );
                a - b
            }
        }
    }
}

// implement pretty printing for units
impl std::fmt::Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Pixels(pixels) => write!(f, "{}px", pixels),
            Size::ScreenWidth(normalised) => write!(f, "{}w", normalised),
            Size::ScreenHeight(normalised) => write!(f, "{}h", normalised),
            Size::Deegrees(degrees) => write!(f, "{}deg", degrees),
            Size::Millimeters(millimeters) => write!(f, "{}mm", millimeters),
            Size::Centimeters(centimeters) => write!(f, "{}cm", centimeters),
            Size::Inches(inches) => write!(f, "{}in", inches),
            Size::Points(points) => write!(f, "{}pt", points),
            Size::Default(default) => write!(f, "{}def", default),
            Size::Quotient(a, b) => write!(f, "({:?})/({:?})", a, b),
            Size::Product(a, b) => write!(f, "({:?})*({:?})", a, b),
            Size::Sum(a, b) => write!(f, "({:?})+({:?})", a, b),
            Size::Difference(a, b) => write!(f, "({:?})-({:?})", a, b),
        }
    }
}

/// Types that can be triangulated, i.e. converted to a list of vertices.
pub trait ToVertices {
    /// Convert the shape to a list of vertices in pixels. The vertices are given as a list of floats,
    /// where each three floats represent the x, y, and z coordinate of a vertex. The z coordinate is
    /// always 0.0. X and y coordinates are given in NDC (Normalized Device Coordinates) space, i.e. between -1
    /// and 1 with the origin in the center of the screen and the point (-1, -1) in the top left corner.
    fn to_vertices_px(
        &self,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Vec<Vertex>;
}

pub struct Rectangle {
    pub left: Size,
    pub top: Size,
    pub width: Size,
    pub height: Size,
}

pub struct Circle {
    pub center_x: Size,
    pub center_y: Size,
    pub radius: Size,
}

impl Rectangle {
    pub fn new(
        left: impl Into<Size>,
        top: impl Into<Size>,
        width: impl Into<Size>,
        height: impl Into<Size>,
    ) -> Self {
        Self {
            left: left.into(),
            top: top.into(),
            width: width.into(),
            height: height.into(),
        }
    }

    pub fn to_pixels(
        &self,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> [f64; 4] {
        let left = self.left.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );
        let top = self.top.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );
        let width = self.width.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );
        let height = self.height.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );

        [left, top, width, height]
    }
}

impl Circle {
    pub fn new(
        center_x: impl Into<Size>,
        center_y: impl Into<Size>,
        radius: impl Into<Size>,
    ) -> Self {
        Self {
            center_x: center_x.into(),
            center_y: center_y.into(),
            radius: radius.into(),
        }
    }
}

impl ToVertices for Rectangle {
    fn to_vertices_px(
        &self,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Vec<Vertex> {
        let left = self.left.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        ) as f32;
        let top = self.top.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        ) as f32;
        let width = self.width.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        ) as f32;
        let height = self.height.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        ) as f32;

        let vertices = vec![
            Vertex {
                position: [left, top, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [left + width, top, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [left + width, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [left, top, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [left + width, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [left, top + height, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        vertices
    }
}

impl ToVertices for Circle {
    fn to_vertices_px(
        &self,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Vec<Vertex> {
        let center_x = self.center_x.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );
        let center_y = self.center_y.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );
        let radius = self.radius.to_pixels(
            width_mm,
            viewing_distance_mm,
            width_px,
            height_px,
        );

        let mut vertices = Vec::new();

        let n_segments = 500;

        for i in 0..n_segments {
            let theta =
                2.0 * std::f64::consts::PI * (i as f64 / n_segments as f64);
            let next_theta = 2.0
                * std::f64::consts::PI
                * ((i + 1) as f64 / n_segments as f64);

            let x = center_x + radius * theta.cos();
            let y = center_y + radius * theta.sin();

            let next_x = center_x + radius * next_theta.cos();
            let next_y = center_y + radius * next_theta.sin();

            vertices.push(Vertex {
                position: [center_x as f32, center_y as f32, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            });
            vertices.push(Vertex {
                position: [x as f32, y as f32, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            });
            vertices.push(Vertex {
                position: [next_x as f32, next_y as f32, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            });
        }

        vertices
    }
}

#[derive(Debug)]
pub enum Transformation2D {
    /// Identity transformation (no transformation).
    Identity,
    /// Rotation around the center of the stimulus.
    RotationCenter(f32),
    /// Rotation around an arbitrary point.
    RotationPoint(f32, Size, Size),
    /// Scale around the center of the stimulus.
    ScaleCenter(f32, f32),
    /// Scale around an arbitrary point.
    ScalePoint(f32, f32, Size, Size),
    /// Shear around the center of the stimulus.
    ShearCenter(f32, f32),
    /// Shear around an arbitrary point.
    ShearPoint(f32, f32, Size, Size),
    /// Translation by x and y.
    Translation(Size, Size),
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

impl Transformation2D {
    /// Convert to the corresponding (homogeneous) 2D transformation matrix.
    #[rustfmt::skip]
    pub fn to_transformation_matrix(&self, width_mm: f64, viewing_distance_mm: f64, width_px: i32, height_px: i32) -> Matrix4<f32> {
        match self {
            Transformation2D::Identity => Matrix4::identity(),
            Transformation2D::RotationCenter(angle) => {
                let angle = angle.to_radians();
                Matrix4::new(
                    angle.cos(), -angle.sin(), 0.0, 0.0,
                    angle.sin(), angle.cos(), 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0., 0., 0., 1.,
                )
            }
            Transformation2D::RotationPoint(angle, x, y) => {
                let angle = angle.to_radians();
                let x = x.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;
                let y = y.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;
                Matrix4::new(
                    angle.cos(), -angle.sin(), 0.0, 0.0,
                    angle.sin(), angle.cos(), 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    x * (1.0 - angle.cos()) + y * angle.sin(), y * (1.0 - angle.cos()) - x * angle.sin(), 0.0, 1.0,
                )
            }
            Transformation2D::ScaleCenter(x, y) => {
                Matrix4::new(
                    *x, 0.0, 0.0, 0.0,
                    0.0, *y, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    (1.0 - x) / 2.0, (1.0 - y) / 2.0, 0.0, 1.0,
                )
            }
            Transformation2D::ScalePoint(x, y, x0, y0) => {
                let x0 = x0.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;

                let y0 = y0.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;

                Matrix4::new(
                    *x, 0.0, 0.0, 0.0,
                    0.0, *y, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    x0 * (1.0 - x), y0 * (1.0 - y), 0.0, 1.0,
                )
            }
            Transformation2D::ShearCenter(x, y) => {
                Matrix4::new(
                    1.0, *x, 0.0, 0.0,
                    *y, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    (1.0 - x) / 2.0, (1.0 - y) / 2.0, 0.0, 1.0,
                )
            }
            Transformation2D::ShearPoint(x, y, x0, y0) => {
                let x0 = x0.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;

                let y0 = y0.to_pixels(
                    width_mm,
                    viewing_distance_mm,
                    width_px,
                    height_px,
                ) as f32;

                Matrix4::new(
                    1.0, *x, 0.0, 0.0,
                    *y, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    x0 * (1.0 - x), y0 * (1.0 - y), 0.0, 1.0,
                )
            }
            Transformation2D::Translation(x, y) => {
                let x = x.to_pixels(1.0, 1.0, 1, 1) as f32;
                let y = y.to_pixels(1.0, 1.0, 1, 1) as f32;
                Matrix4::new(
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0., 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    x, y, 0.0, 1.0,
                )
            }
            Transformation2D::Matrix(matrix) => {
                let mut matrix = matrix.clone();
                matrix[(0, 3)] = 0.0;
                matrix[(1, 3)] = 0.0;
                matrix[(2, 3)] = 0.0;
                matrix[(3, 3)] = 1.0;
                matrix.to_homogeneous()
            }
            Transformation2D::Homogeneous(matrix) => matrix.clone(),
            Transformation2D::Product(a,b) =>
            {
                let a = a.to_transformation_matrix(width_mm, viewing_distance_mm, width_px, height_px);
                let b = b.to_transformation_matrix(width_mm, viewing_distance_mm, width_px, height_px);
                a * b
            }
        }
    }
}

/// A struct that represents a vertex in a 3D space. A vertex consists of a position, a color, and texture coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>()
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
pub trait GetCenter {
    /// Get the x and y coordinates of the center of the stimulus. This is used to calculate the
    /// transformations of the stimulus, such `RotationCenter`.
    ///
    /// # Returns
    /// A tuple containing the x and y coordinates of the center of the stimulus.
    fn get_center(&self) -> (Size, Size);
}

// impl GetCenter for Rectangle {
//     fn get_center(&self) -> UnitPoint2D {
//         let left = self.left.to_pixels(1.0, 1.0, 1, 1);
//         let top = self.top.to_pixels(1.0, 1.0, 1, 1);
//         let width = self.width.to_pixels(1.0, 1.0, 1, 1);
//         let height = self.height.to_pixels(1.0, 1.0, 1, 1);

//         (left + width / 2.0, top + height / 2.0)
//     }
