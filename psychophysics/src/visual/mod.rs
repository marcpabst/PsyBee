pub mod color;
pub mod gratings;
pub mod pwindow;
pub mod shape;
pub mod text;

use wgpu::{Device, Queue, SurfaceConfiguration};

// re-export the color module
pub use color::Color;

/// A trait for renderable objects that can be drawn to the screen.
pub trait Renderable {
    /// Prepare the renderable object for rendering.
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
    ) -> ();
    /// Render the object to the screen.
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
    fn is_finnished(&self) -> bool {
        false
    }
}

pub enum Scaling {
    Pixel,                             // all units are in pixels
    Normalized,                        // units are scaled to the window size
    ScaleFactor { scale_factor: f32 }, // units are scaled by a factor
    Degree,                            // units are in degrees of visual angle
    Default,                           // use the window's default scaling
}
