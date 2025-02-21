// Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Origin of the coordinate system.
    pub coordinate_origin: CoordinateOrigin,
}

pub enum CoordinateOrigin {
    TopLeft,
    Center,
}
