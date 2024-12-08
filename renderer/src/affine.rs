use shapes::Point;

use super::shapes;

#[derive(Debug, Clone, Copy)]
pub struct Affine(pub [f64; 6]);

impl Affine {
    #[inline]
    pub const fn identity() -> Self {
        Self::scale(1.0)
    }

    #[inline]
    pub const fn scale(s: f64) -> Affine {
        Affine([s, 0.0, 0.0, s, 0.0, 0.0])
    }

    #[inline]
    pub const fn scale_xy(sx: f64, sy: f64) -> Affine {
        Affine([sx, 0.0, 0.0, sy, 0.0, 0.0])
    }

    #[inline]
    pub fn scale_xy_at(sx: f64, sy: f64, x: f64, y: f64) -> Affine {
       // we need to translate the point to the origin, scale it, and then translate it back
        Affine::translate(x, y) * Affine::scale_xy(sx, sy) * Affine::translate(-x, -y)
    }

    #[inline]
    pub const fn translate(x: f64, y: f64) -> Affine {
        Affine([1.0, 0.0, 0.0, 1.0, x, y])
    }

    #[inline]
    pub fn rotate(theta: f64) -> Affine {
        let (s, c) = theta.sin_cos();
        Affine([c, s, -s, c, 0.0, 0.0])
    }

    #[inline]
    pub fn rotate_at(theta: f64, x: f64, y: f64) -> Affine {
        let (s, c) = theta.sin_cos();
        Affine([c, s, -s, c, x - x * c + y * s, y - x * s - y * c])
    }
}

impl std::ops::Mul for Affine {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = self.0[0] * rhs.0[0] + self.0[1] * rhs.0[2];
        let b = self.0[0] * rhs.0[1] + self.0[1] * rhs.0[3];
        let c = self.0[2] * rhs.0[0] + self.0[3] * rhs.0[2];
        let d = self.0[2] * rhs.0[1] + self.0[3] * rhs.0[3];
        let e = self.0[4] * rhs.0[0] + self.0[5] * rhs.0[2] + rhs.0[4];
        let f = self.0[4] * rhs.0[1] + self.0[5] * rhs.0[3] + rhs.0[5];
        Affine([a, b, c, d, e, f])
    }
}

// [f64; 6] 3x3 array into Affine
impl Into<Affine> for [f64; 6] {
    fn into(self) -> Affine {
        Affine(self)
    }
}

// from nalgebra::Matrix3<f32>
impl Into<Affine> for nalgebra::Matrix3<f32> {
    fn into(self) -> Affine {
        Affine([
            self[(0, 0)] as f64,
            self[(0, 1)] as f64,
            self[(1, 0)] as f64,
            self[(1, 1)] as f64,
            self[(0, 2)] as f64,
            self[(1, 2)] as f64,
        ])
    }
}
