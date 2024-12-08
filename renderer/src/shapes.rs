#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub trait Shape: Clone {}

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub a: Point,
    pub b: Point,
}

#[derive(Debug, Clone)]
pub struct RoundedRectangle {
    pub a: Point,
    pub b: Point,
    pub radius: f64,
}

impl Shape for Circle {}
impl Shape for Rectangle {}
impl Shape for RoundedRectangle {}
