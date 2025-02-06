pub use super::scenes::Scene;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Into<Point> for (f64, f64) {
    fn into(self) -> Point {
        Point { x: self.0, y: self.1 }
    }
}

#[derive(Debug, Clone)]
pub enum Shape {
    Circle {
        center: Point,
        radius: f64,
    },
    Rectangle {
        a: Point,
        b: Point,
    },
    RoundedRectangle {
        a: Point,
        b: Point,
        radius: f64,
    },
    Line {
        start: Point,
        end: Point,
    },
    Ellipse {
        center: Point,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
}

impl Shape {
    pub fn circle(center: impl Into<Point>, radius: f64) -> Self {
        Self::Circle {
            center: center.into(),
            radius,
        }
    }

    pub fn rectangle(topleft: impl Into<Point>, width: f64, height: f64) -> Self {
        let a = topleft.into();
        let width = width;
        let height = height;
        Self::Rectangle {
            a,
            b: (a.x + width, a.y + height).into(),
        }
    }

    pub fn rounded_rectangle(topleft: impl Into<Point>, width: f64, height: f64, radius: f64) -> Self {
        let a = topleft.into();
        let width = width;
        let height = height;
        Self::RoundedRectangle {
            a,
            b: (a.x + width, a.y + height).into(),
            radius,
        }
    }

    pub fn line(start: impl Into<Point>, end: impl Into<Point>) -> Self {
        Self::Line {
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn ellipse(center: impl Into<Point>, radius_x: f64, radius_y: f64, rotation: f64) -> Self {
        Self::Ellipse {
            center: center.into(),
            radius_x,
            radius_y,
            rotation,
        }
    }
}
