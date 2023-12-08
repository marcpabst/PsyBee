pub mod gratings;
pub mod pwindow;
pub mod shape;
pub mod text;
use wgpu::{Device, Queue, SurfaceConfiguration};

// Renderable trait should be implemented by all visual stimuli
// the API is extremely simple: render() and update() and follows the
// the middlewares pattern used by wgpu
pub trait Renderable {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> ();
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
    fn is_finnished(&self) -> bool {
        false
    }
}

// enum to represent a color
#[derive(Debug, Copy, Clone)]
pub enum Color {
    RGB { r: f64, g: f64, b: f64 },
    RGBA { r: f64, g: f64, b: f64, a: f64 },
}

// allow for conversion from (u8, u8, u8) to RGB
impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::RGB {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
}

// allow for conversion to wgpu::Color
impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        match self {
            Color::RGB { r, g, b } => wgpu::Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
                a: 1.0,
            },
            Color::RGBA { r, g, b, a } => wgpu::Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
                a: a as f64,
            },
        }
    }
}

// alow conversion into glyphon::Color
impl Into<glyphon::Color> for Color {
    fn into(self) -> glyphon::Color {
        match self {
            Color::RGB { r, g, b } => {
                glyphon::Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
            }
            Color::RGBA { r, g, b, a } => glyphon::Color::rgba(
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            ),
        }
    }
}
