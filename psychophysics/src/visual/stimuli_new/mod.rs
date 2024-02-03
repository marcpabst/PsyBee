use super::Window;

pub mod base;

/// The stimulus trait.
pub trait Stimulus: Send + Sync + Clone {
    /// Prepare the renderable object for rendering.
    fn prepare(&mut self, window: &Window) -> ();
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
