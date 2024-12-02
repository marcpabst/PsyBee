//! This module contains structs and traits that are used to specify the
//! geometry of a stimulus. This includes rectangles, circles, and
//! transformations.

use nalgebra::{Matrix3, Vector3};
use pyo3::{prelude::*, PyClass};

#[pyclass]
#[derive(Clone, Debug)]
pub struct BoxedSize(Box<Size>);

impl BoxedSize {
    pub fn new(size: Size) -> Self {
        BoxedSize(Box::new(size))
    }
}

impl std::ops::Deref for BoxedSize {
    type Target = Size;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Can either be a dimensionless number or a size.
pub enum NumberOrSize {
    Dimensionless(f32),
    Size(Size),
}

impl NumberOrSize {
    pub fn eval(&self, props: &super::window::WindowPhysicalProperties) -> f32 {
        match self {
            NumberOrSize::Dimensionless(value) => *value,
            NumberOrSize::Size(size) => size.eval(props),
        }
    }
}

/// This enum is used to specify the size and position of a stimulus. The unit
/// can be specified in different ways, which will be evaluated just before the
/// object is rendered. This allows for the size of the object to be specified
/// in a flexible way, e.g. as a fraction of the screen size or in degrees of
/// visual angle.
///
/// Important: The unit is specified in the constructor of the object, but its
/// actual size in pixels is only calculated when the object is rendered. This
/// is because the size of the object depends on the size of the window, the
/// distance of the observer to the screen, and the physical size of the screen.
/// As some of these parameters may change during the experiment, the size and
/// position of the object in pixels can only be known at the time of rendering.
///
/// # Examples
///
/// ```
/// use psybee::visual::geometry::Size;
///
/// // create a unit that is 100 pixels wide
/// let unit = Size::Pixels(100.0);
///
/// // create a unit that is 10% of the screen width
/// let unit = Size::ScreenWidth(0.1);
///
/// // create a unit that is 10% of the screen height
/// let unit = Size::ScreenHeight(0.1);
/// ```
#[derive(Clone, Debug)]
#[pyclass]
pub enum Size {
    // Physical pixels
    Pixels(f32),
    /// Fraction of the screen height.
    ScreenHeight(f32),
    /// Fraction of the screen width.
    ScreenWidth(f32),
    /// Degrees of visual angle.
    Degrees(f32),
    /// Millimeters.
    Millimeters(f32),
    /// Centimeters.
    Centimeters(f32),
    /// Inches.
    Inches(f32),
    /// Points.
    Points(f32),
    /// Qutioent of a unit and a float (the Unit divided by the float).
    Quotient(BoxedSize, f32),
    /// Product of a unit and a float (the Unit multiplied by the float).
    Product(BoxedSize, f32),
    /// Sum of two units
    Sum(BoxedSize, BoxedSize),
    /// Difference of two units
    Difference(BoxedSize, BoxedSize),
}

pub struct IntoSize(pub Size);

impl From<IntoSize> for Size {
    fn from(value: IntoSize) -> Self {
        value.0
    }
}

impl<'py> FromPyObject<'py> for IntoSize {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // try to extract a Size
        if let Ok(value) = ob.extract::<Size>() {
            return Ok(IntoSize(value));
        }
        // try to extract a float (-> Pixels)
        else if let Ok(value) = ob.extract::<f32>() {
            return Ok(IntoSize(Size::Pixels(value)));
        } else if let Ok(value) = ob.extract::<i32>() {
            return Ok(IntoSize(Size::Pixels(value as f32)));
        }
        // try to extract a string and use a regex to parse it
        else if let Ok(value) = ob.extract::<String>() {
            Ok(IntoSize(Size::from_str(&value).unwrap()))
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Size must be a float."));
        }
    }
}

#[derive(Clone, Debug)]
pub struct SizeVector2D {
    pub x: Size,
    pub y: Size,
}

pub type Position = SizeVector2D;
pub type Size2D = SizeVector2D;

impl From<(f64, f64)> for SizeVector2D {
    fn from((x, y): (f64, f64)) -> Self {
        SizeVector2D {
            x: Size::new_default(x as f32),
            y: Size::new_default(y as f32),
        }
    }
}

impl From<(f32, f32)> for SizeVector2D {
    fn from((x, y): (f32, f32)) -> Self {
        SizeVector2D {
            x: Size::new_default(x as f32),
            y: Size::new_default(y as f32),
        }
    }
}

impl From<(Size, Size)> for SizeVector2D {
    fn from((x, y): (Size, Size)) -> Self {
        SizeVector2D { x, y }
    }
}

impl From<f32> for Size {
    /// Convert from a float to a unit. The float is interpreted as a number of
    /// `Default` units.
    fn from(f: f32) -> Self {
        Size::new_default(f)
    }
}

impl From<i64> for Size {
    /// Convert from an integer to a unit. The integer is interpreted as a
    /// number of `Default` units.
    fn from(i: i64) -> Self {
        Size::new_default(i as f32)
    }
}

impl From<f64> for Size {
    /// Convert from a float to a unit. The float is interpreted as a number of
    /// `Default` units.
    fn from(f: f64) -> Self {
        Size::new_default(f as f32)
    }
}

impl std::ops::Add for Size {
    type Output = Size;

    /// Add two units together. The results is a `Unit::Sum`.
    fn add(self, rhs: Self) -> Self::Output {
        Size::Sum(BoxedSize::new(self), BoxedSize::new(rhs))
    }
}

impl std::ops::Sub for Size {
    type Output = Size;

    /// Subtract two units. The results is a `Unit::Difference`.
    fn sub(self, rhs: Self) -> Self::Output {
        Size::Difference(BoxedSize::new(self), BoxedSize::new(rhs))
    }
}

impl std::ops::Mul<f32> for Size {
    type Output = Size;

    /// Multiply two units. The results is a `Unit::Product`.
    fn mul(self, rhs: f32) -> Self::Output {
        Size::Product(BoxedSize::new(self), rhs)
    }
}

impl std::ops::Div<f32> for Size {
    type Output = Size;

    /// Divide two units. The results is a `Unit::Quotient`.
    fn div(self, rhs: f32) -> Self::Output {
        Self::Quotient(BoxedSize::new(self), rhs)
    }
}

// implements the minus operator for a single size
impl std::ops::Neg for Size {
    type Output = Size;

    /// Negate a unit. The results is a `Unit::Product` with a factor of -1.0.
    fn neg(self) -> Self::Output {
        Size::Product(BoxedSize::new(self), -1.0)
    }
}

// pub trait ToPixels {
//     type Output;

//     fn to_pixels(&self, screenwidth_mm: f64, viewing_distance_mm: f64, width_px: u32, height_px: u32) -> Self::Output;
// }

impl Size {
    /// Convert the given angle of visual angle to millimeters, taking the
    /// viewing distance into account.
    ///
    /// # Arguments
    ///
    /// * `angle` - The angle in degrees.
    /// * `viewing_distance_mm` - The viewing distance in millimeters.
    ///
    /// # Returns
    ///
    /// The distance in millimeters.
    fn angle_to_milimeter(angle: f32, viewing_distance_mm: f32) -> Size {
        Size::Millimeters(2.0 * viewing_distance_mm * (angle.to_radians() / 2.0).tan())
    }

    /// Convert the given unit to pixels, taking the physical size of the screen
    /// and the viewing distance into account.
    ///
    /// # Arguments
    ///
    /// * `props` - The physical properties of the window.
    ///
    /// # Returns
    ///
    /// The unit converted to pixels (as a float).
    pub fn eval(&self, props: &super::window::WindowPhysicalProperties) -> f32 {
        match self {
            Size::Pixels(pixels) => *pixels,
            Size::ScreenWidth(normalised) => *normalised * props.width as f32,
            Size::ScreenHeight(normalised) => *normalised * props.height as f32,
            Size::Degrees(degrees) => Size::angle_to_milimeter(*degrees, props.viewing_distance).eval(props),
            Size::Millimeters(millimeters) => *millimeters / 1000.0 * props.width as f32 / props.width_m,
            Size::Centimeters(centimeters) => Size::Millimeters(*centimeters * 10.0).eval(props),
            Size::Inches(inches) => Size::Millimeters(*inches * 25.4).eval(props),
            Size::Points(points) => Size::Inches(*points / 72.0).eval(props),
            Size::Quotient(a, b) => {
                // first, we resolve `a` to pixels, the we divide by b
                let a = a.eval(props);
                a / b
            }
            Size::Product(a, b) => {
                // first, we resolve `a` to pixels, the we multiply with b
                let a = a.eval(props);
                a * b
            }
            Size::Sum(a, b) => {
                let a = a.eval(props);
                let b = b.eval(props);
                a + b
            }
            Size::Difference(a, b) => {
                let a = a.eval(props);
                let b = b.eval(props);
                a - b
            }
        }
    }

    /// Create a new `Size` with the given value in the default unit (pixels).
    pub fn new_default(value: f32) -> Size {
        Size::Pixels(value)
    }

    pub fn from_str(string: &str) -> Result<Size, String> {
        let string = string.trim();
        // check if negative
        let negative = string.starts_with('-');
        let string = if negative { &string[1..] } else { string };

        // extract the number and the unit
        let (number, unit) = string.split_at(
            string
                .find(|c: char| !c.is_digit(10) && c != '.')
                .unwrap_or(string.len()),
        );
        let number = number.parse::<f32>().map_err(|e| e.to_string())?;
        let unit = unit.trim();

        // match the unit
        match unit {
            "px" => Ok(Size::Pixels(if negative { -number } else { number })),
            "sw" => Ok(Size::ScreenWidth(if negative { -number } else { number })),
            "sh" => Ok(Size::ScreenHeight(if negative { -number } else { number })),
            "deg" => Ok(Size::Degrees(if negative { -number } else { number })),
            "mm" => Ok(Size::Millimeters(if negative { -number } else { number })),
            "cm" => Ok(Size::Centimeters(if negative { -number } else { number })),
            "in" => Ok(Size::Inches(if negative { -number } else { number })),
            "pt" => Ok(Size::Points(if negative { -number } else { number })),
            _ => Err(format!("Unknown unit: {}", unit)),
        }
    }
}

#[pymethods]
impl Size {
    // constructors
    #[new]
    fn __new__(string: String) -> PyResult<Size> {
        let size = Size::from_str(&string).unwrap();
        Ok(size)
    }

    // printing
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    // addition
    fn __add__(&self, other: &Size) -> Size {
        self.clone() + other.clone()
    }

    // subtraction
    fn __sub__(&self, other: &Size) -> Size {
        self.clone() - other.clone()
    }

    // negation
    fn __neg__(&self) -> Size {
        -self.clone()
    }

    // evaluation
    #[pyo3(name = "eval")]
    fn py_eval(&self, window: &super::window::WrappedWindow) -> f32 {
        let props = window.inner().physical_properties;
        self.eval(&props)
    }
}

impl SizeVector2D {
    /// Convert the point to pixels, taking the physical size of the screen and
    /// the viewing distance into account.
    ///
    /// # Arguments
    ///
    /// * `screenwidth_mm` - The width of the screen in millimeters.
    /// * `width_px` - The width of the screen in pixels.
    /// * `viewing_distance_mm` - The viewing distance in millimeters.
    /// * `height_px` - The height of the screen in pixels.
    ///
    /// # Returns
    ///
    /// The point converted to pixels.
    fn to_pixels(&self, props: &super::window::WindowPhysicalProperties) -> (f32, f32) {
        (self.x.eval(props), self.y.eval(props))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct BoxedTransformation2D(Box<Transformation2D>);

impl BoxedTransformation2D {
    pub fn new(transformation: Transformation2D) -> Self {
        BoxedTransformation2D(Box::new(transformation))
    }
}

impl std::ops::Deref for BoxedTransformation2D {
    type Target = Transformation2D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 2D transformations that can be applied to a stimulus.
/// This enum is used to specify the transformation of a stimulus. The
/// transformation is applied to the object just before it is rendered.
///
/// Important: The transformation is specified in the constructor of the object,
/// but its actual transformation matrix is only calculated when the object is
/// rendered. This is because the transformation of the object depends on the
/// size of the window, the distance of the observer to the screen, and the
/// physical size of the screen. As some of these parameters may change during
/// the experiment, the transformation matrix of the object can only be known at
/// the time of rendering.
#[derive(Debug, Clone)]
#[pyclass]
pub enum Transformation2D {
    /// Identity transformation (no transformation).
    Identity(),
    /// Rotation around the center of the object.
    RotationCenter(f32),
    /// Rotation around an arbitrary point.
    RotationPoint(f32, Size, Size),
    /// Scale around the center of the object.
    ScaleCenter(f32, f32),
    /// Scale around an arbitrary point.
    ScalePoint(f32, f32, Size, Size),
    /// Shear around the center of the object.
    ShearCenter(f32, f32),
    /// Shear around an arbitrary point.
    ShearPoint(f32, f32, Size, Size),
    /// Translation by x and y.
    Translation(Size, Size),
    /// Arbitrary 2D transformation matrix.
    // Matrix(Matrix2<f32>),
    // /// Homogeneous 2D transformation matrix. This 4x4 matrix will be applied to
    // /// the coordinates in NDC (Normalized Device Coordinates) space, but
    // /// please note that the specific coordinate system this matrix will be
    // /// applied to is considered an implementation detail and may change in
    // /// the future. It is recommended to use the other variants instead or
    // /// to combine a 2D transformation matrix with a `Translation`
    // /// transformation, which will take care of the coordinate system for
    // /// you.
    // Homogeneous(Matrix3<f32>),
    /// Product of two transformations. This variant is used to combine multiple
    /// transformations in a lazy way.
    Product(BoxedTransformation2D, BoxedTransformation2D),
}

impl Transformation2D {
    #[allow(non_snake_case)]
    pub fn translation(x: impl Into<Size>, y: impl Into<Size>) -> Transformation2D {
        Transformation2D::Translation(x.into(), y.into())
    }

    /// Convert to the corresponding (homogeneous) 2D transformation matrix.
    #[rustfmt::skip]
    pub fn eval(&self, props: &super::window::WindowPhysicalProperties) -> Matrix3<f32> {
        match self {
            Transformation2D::Identity() => Matrix3::identity(),
            Transformation2D::RotationCenter(_angle) => {
                todo!()
            }
            Transformation2D::RotationPoint(angle, x, y) => {
                let angle = angle.to_radians();
                let x = x.eval(
                    props
                ) as f32;
                let y = y.eval(
                    props
                ) as f32;

                Matrix3::new(
                    angle.cos(), -angle.sin(), 0.0,
                    angle.sin(), angle.cos(), 0.0,
                    x * (1.0 - angle.cos()) + y * angle.sin(), y * (1.0 - angle.cos()) - x * angle.sin(), 1.0,
                )
            }
            Transformation2D::ScaleCenter(_x, _y) => {
               todo!()
            }
            Transformation2D::ScalePoint(x, y, x0, y0) => {
                let x0 = x0.eval(props);
                let y0 = y0.eval(props);

                Matrix3::new(
                    *x, 0.0, 0.0,
                    0.0, *y, 0.0,
                    x0 * (1.0 - x), y0 * (1.0 - y), 1.0,
                )
            }
            Transformation2D::ShearCenter(_x, _y) => {
                todo!()
            }
            Transformation2D::ShearPoint(x, y, x0, y0) => {
                let x0 = x0.eval(props);
                let y0 = y0.eval(props);

                Matrix3::new(
                    1.0, *x, 0.0,
                    *y, 1.0, 0.0,
                    -x0 * y, -y0 * x, 1.0,
                )
            }
            Transformation2D::Translation(x, y) => {
                let x = x.eval(props) as f32;
                let y = y.eval(props) as f32;

                Matrix3::new(
                    1.0, 0.0, 0.0,
                    0.0, 1.0, 0.0,
                    x, y, 1.0,
                )
            }
            // Transformation2D::Matrix(matrix) => {
            //     matrix.clone().to_homogeneous()
            // }
            //Transformation2D::Homogeneous(matrix) => matrix.clone(),
            Transformation2D::Product(a,b) =>
            {
                let a = a.eval(props);
                let b = b.eval(props);
                a * b
            }
        }
    }

    pub fn transform_point(&self, x: f32, y: f32, props: &super::window::WindowPhysicalProperties) -> (f32, f32) {
        let matrix = self.eval(props).transpose();
        let newpoint = matrix * Vector3::new(x, y, 1.0);
        (newpoint.x, newpoint.y)
    }
}

// allow multiplication of transformations
impl std::ops::Mul for Transformation2D {
    type Output = Transformation2D;

    fn mul(self, rhs: Self) -> Self::Output {
        Transformation2D::Product(BoxedTransformation2D::new(self), BoxedTransformation2D::new(rhs))
    }
}

// basic 2d shapes
#[derive(Debug, Clone)]
#[pyclass]
pub enum Shape {
    /// A rectangle.
    Rectangle {
        x: Size,
        y: Size,
        width: Size,
        height: Size,
    },

    /// A circle.
    Circle { x: Size, y: Size, radius: Size },

    /// A line.
    Line { x1: Size, y1: Size, x2: Size, y2: Size },

    /// An ellipse.
    Ellipse {
        x: Size,
        y: Size,
        radius_x: Size,
        radius_y: Size,
    },

    /// A polygon.
    Polygon {
        points: Vec<(Size, Size)>,
    },
}


#[pymethods]
impl Shape {
    #[staticmethod]
    /// Create a new rectangle.
    fn rectangle(x: IntoSize, y: IntoSize, width: IntoSize, height: IntoSize) -> Shape {
        Shape::Rectangle {
            x: x.into(),
            y: y.into(),
            width: width.into(),
            height: height.into(),
        }
    }

    #[staticmethod]
    /// Create a new circle.
    fn circle(x: IntoSize, y: IntoSize, radius: IntoSize) -> Shape {
        Shape::Circle {
            x: x.into(),
            y: y.into(),
            radius: radius.into(),
        }
    }

    #[staticmethod]
    /// Create a new line.
    fn line(x1: IntoSize, y1: IntoSize, x2: IntoSize, y2: IntoSize) -> Shape {
        Shape::Line {
            x1: x1.into(),
            y1: y1.into(),
            x2: x2.into(),
            y2: y2.into(),
        }
    }

    #[staticmethod]
    /// Create a new ellipse.
    fn ellipse(x: IntoSize, y: IntoSize, radius_x: IntoSize, radius_y: IntoSize) -> Shape {
        Shape::Ellipse {
            x: x.into(),
            y: y.into(),
            radius_x: radius_x.into(),
            radius_y: radius_y.into(),
        }
    }

    // #[staticmethod]
    // /// Create a new pentagon.
    // fn pentagon(x: IntoSize, y: IntoSize, radius: IntoSize) -> Shape {
    //     let radius: Size = radius.into();
    //     let x: Size = x.into();
    //     let y: Size = y.into();
    //     let mut points = Vec::new();
    //
    //     for i in 0..5 {
    //         let angle = 2.0 * std::f32::consts::PI / 5.0 * i as f32;
    //         let rotation = Transformation2D::RotationPoint(angle, x.clone(), y.clone());
    //         let x = &x + &radius;
    //         let y = &y + &radius;
    //
    //         points.push((x, y));
    //     }
    //     Shape::Polygon { points }
    // }

    // for printing
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
