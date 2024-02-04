use crate::GPUState;

use super::{window::WindowState, Window};

pub mod base_stimulus;
pub mod color_stimulus;
pub mod gratings_stimulus;

pub use color_stimulus::ColorStimulus;
pub use gratings_stimulus::GratingType;
pub use gratings_stimulus::GratingsStimulus;

/// The stimulus trait.
pub trait Stimulus: Send + Sync {
    /// Prepare the renderable object for rendering.
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) -> ();
    /// Render the object to the screen.
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
}
