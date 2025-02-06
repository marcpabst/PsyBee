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
    pub fn skew_xy(sx: f64, sy: f64) -> Affine {
        Affine([1.0, sy.tan(), sx.tan(), 1.0, 0.0, 0.0])
    }

    #[inline]
    pub const fn translate(x: f64, y: f64) -> Affine {
        Affine([1.0, 0.0, 0.0, 1.0, x, y])
    }

    #[inline]
    pub fn rotate(theta: f64) -> Affine {
        let theta = theta.to_radians();
        let (s, c) = theta.sin_cos();
        Affine([c, s, -s, c, 0.0, 0.0])
    }

    #[inline]
    pub fn rotate_at(theta: f64, x: f64, y: f64) -> Affine {
        let (s, c) = theta.sin_cos();
        Affine([c, s, -s, c, x - x * c + y * s, y - x * s - y * c])
    }

    #[inline]
    pub fn pre_translate(&mut self, x: f64, y: f64) {
        self.0[4] += x * self.0[0] + y * self.0[2];
        self.0[5] += x * self.0[1] + y * self.0[3];
    }

    #[inline]
    pub fn pre_scale(&mut self, sx: f64, sy: f64) {
        self.0[0] *= sx;
        self.0[1] *= sy;
        self.0[2] *= sx;
        self.0[3] *= sy;
    }

    #[inline]
    pub fn pre_rotate(&mut self, theta: f64) {
        let (s, c) = theta.sin_cos();
        let a = self.0[0] * c + self.0[2] * s;
        let b = self.0[1] * c + self.0[3] * s;
        let c = self.0[0] * -s + self.0[2] * c;
        let d = self.0[1] * -s + self.0[3] * c;
        self.0[0] = a;
        self.0[1] = b;
        self.0[2] = c;
        self.0[3] = d;
    }

    #[inline]
    pub fn pre_skew(&mut self, sx: f64, sy: f64) {
        let a = self.0[0] + self.0[2] * sy;
        let b = self.0[1] + self.0[3] * sy;
        let c = self.0[0] * sx + self.0[2];
        let d = self.0[1] * sx + self.0[3];
        self.0[0] = a;
        self.0[1] = b;
        self.0[2] = c;
        self.0[3] = d;
    }

    #[inline]
    pub fn post_translate(&mut self, x: f64, y: f64) {
        self.0[4] += x;
        self.0[5] += y;
    }

    #[inline]
    pub fn post_scale(&mut self, sx: f64, sy: f64) {
        self.0[0] *= sx;
        self.0[1] *= sx;
        self.0[2] *= sy;
        self.0[3] *= sy;
    }

    #[inline]
    pub fn post_rotate(&mut self, theta: f64) {
        let (s, c) = theta.sin_cos();
        let a = self.0[0] * c + self.0[1] * s;
        let b = self.0[0] * -s + self.0[1] * c;
        let c = self.0[2] * c + self.0[3] * s;
        let d = self.0[2] * -s + self.0[3] * c;
        self.0[0] = a;
        self.0[1] = b;
        self.0[2] = c;
        self.0[3] = d;
    }

    #[inline]
    pub fn post_skew(&mut self, sx: f64, sy: f64) {
        let a = self.0[0] + self.0[1] * sx;
        let b = self.0[0] * sy + self.0[1];
        let c = self.0[2] + self.0[3] * sx;
        let d = self.0[2] * sy + self.0[3];
        self.0[0] = a;
        self.0[1] = b;
        self.0[2] = c;
        self.0[3] = d;
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
