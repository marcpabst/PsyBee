pub mod geometry;
pub mod pwindow;
pub mod stimuli;
pub mod text;

use wgpu::{Device, Queue, SurfaceConfiguration};

// re-export the color module
pub use color::Color;

use self::geometry::Size;

/// A trait for renderable objects that can be drawn to the screen.
pub trait Renderable {
    /// Prepare the renderable object for rendering.
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &pwindow::WindowHandle,
    ) -> ();
    /// Render the object to the screen.
    fn render(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> ();
    fn is_finnished(&self) -> bool {
        false
    }
}
