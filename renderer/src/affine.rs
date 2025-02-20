#[derive(Debug, Clone, Copy)]
pub struct Affine(nalgebra::Matrix3<f32>);

impl Affine {
    pub fn from_matrix(matrix: nalgebra::Matrix3<f32>) -> Self {
        Affine(matrix)
    }

    pub fn as_matrix(&self) -> nalgebra::Matrix3<f32> {
        self.0
    }

    #[inline]
    pub const fn identity() -> Self {
        Self::scale(1.0)
    }

    #[inline]
    pub const fn scale(s: f64) -> Affine {
        Affine(nalgebra::Matrix3::new(
            s as f32, 0.0, 0.0, 0.0, s as f32, 0.0, 0.0, 0.0, 1.0,
        ))
    }

    #[inline]
    pub const fn scale_xy(sx: f64, sy: f64) -> Affine {
        Affine(nalgebra::Matrix3::new(
            sx as f32, 0.0, 0.0, 0.0, sy as f32, 0.0, 0.0, 0.0, 1.0,
        ))
    }

    #[inline]
    pub fn scale_xy_at(sx: f64, sy: f64, x: f64, y: f64) -> Affine {
        let mut matrix = nalgebra::Matrix3::identity();
        matrix[(0, 0)] = sx as f32;
        matrix[(1, 1)] = sy as f32;
        matrix[(0, 2)] = x as f32 * (1.0 - sx as f32);
        matrix[(1, 2)] = y as f32 * (1.0 - sy as f32);
        Affine(matrix)
    }

    #[inline]
    pub fn skew_xy(sx: f64, sy: f64) -> Affine {
        let mut matrix = nalgebra::Matrix3::identity();
        matrix[(0, 1)] = sx as f32;
        matrix[(1, 0)] = sy as f32;
        Affine(matrix)
    }

    #[inline]
    pub const fn translate(x: f64, y: f64) -> Affine {
        Affine(nalgebra::Matrix3::new(
            1.0, 0.0, x as f32, 0.0, 1.0, y as f32, 0.0, 0.0, 1.0,
        ))
    }

    #[inline]
    pub fn rotate(theta: f64) -> Affine {
        let theta = theta.to_radians();
        let mut matrix = nalgebra::Matrix3::identity();
        let (s, c) = theta.sin_cos();
        matrix[(0, 0)] = c as f32;
        matrix[(0, 1)] = -s as f32;
        matrix[(1, 0)] = s as f32;
        matrix[(1, 1)] = c as f32;
        Affine(matrix)
    }

    #[inline]
    pub fn rotate_at(theta: f64, x: f64, y: f64) -> Affine {
        let theta = theta.to_radians();
        let mut matrix = nalgebra::Matrix3::identity();
        let (s, c) = theta.sin_cos();
        matrix[(0, 0)] = c as f32;
        matrix[(0, 1)] = -s as f32;
        matrix[(1, 0)] = s as f32;
        matrix[(1, 1)] = c as f32;
        matrix[(0, 2)] = x as f32 * (1.0 - c as f32) + y as f32 * s as f32;
        matrix[(1, 2)] = y as f32 * (1.0 - c as f32) - x as f32 * s as f32;
        Affine(matrix)
    }

    #[inline]
    pub fn pre_translate(&mut self, x: f64, y: f64) {
        let translation = nalgebra::Matrix3::new(1.0, 0.0, x as f32, 0.0, 1.0, y as f32, 0.0, 0.0, 1.0);
        self.0 = translation * self.0;
    }

    #[inline]
    pub fn pre_scale(&mut self, sx: f64, sy: f64) {
        let scale = nalgebra::Matrix3::new(sx as f32, 0.0, 0.0, 0.0, sy as f32, 0.0, 0.0, 0.0, 1.0);
        self.0 = scale * self.0;
    }

    #[inline]
    pub fn pre_rotate(&mut self, theta: f64) {
        let theta = theta.to_radians();
        let (s, c) = theta.sin_cos();
        let rotation = nalgebra::Matrix3::new(c as f32, -s as f32, 0.0, s as f32, c as f32, 0.0, 0.0, 0.0, 1.0);
        self.0 = rotation * self.0;
    }

    #[inline]
    pub fn pre_skew(&mut self, sx: f64, sy: f64) {
        let skew = nalgebra::Matrix3::new(1.0, sx as f32, 0.0, sy as f32, 1.0, 0.0, 0.0, 0.0, 1.0);
        self.0 = skew * self.0;
    }

    #[inline]
    pub fn post_translate(&mut self, x: f64, y: f64) {
        let translation = nalgebra::Matrix3::new(1.0, 0.0, x as f32, 0.0, 1.0, y as f32, 0.0, 0.0, 1.0);
        self.0 = self.0 * translation;
    }

    #[inline]
    pub fn post_scale(&mut self, sx: f64, sy: f64) {
        let scale = nalgebra::Matrix3::new(sx as f32, 0.0, 0.0, 0.0, sy as f32, 0.0, 0.0, 0.0, 1.0);
        self.0 = self.0 * scale;
    }

    #[inline]
    pub fn post_rotate(&mut self, theta: f64) {
        let theta = theta.to_radians();
        let (s, c) = theta.sin_cos();
        let rotation = nalgebra::Matrix3::new(c as f32, -s as f32, 0.0, s as f32, c as f32, 0.0, 0.0, 0.0, 1.0);
        self.0 = self.0 * rotation;
    }

    #[inline]
    pub fn post_skew(&mut self, sx: f64, sy: f64) {
        let skew = nalgebra::Matrix3::new(1.0, sx as f32, 0.0, sy as f32, 1.0, 0.0, 0.0, 0.0, 1.0);
        self.0 = self.0 * skew;
    }
}

impl std::ops::Mul for Affine {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Affine(self.0 * rhs.0)
    }
}

// from nalgebra::Matrix3<f32>
impl Into<Affine> for nalgebra::Matrix3<f32> {
    fn into(self) -> Affine {
        Affine(self)
    }
}
