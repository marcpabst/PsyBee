use crate::GPUState;

use super::{window::WindowState, Window};

pub mod base_stimulus;
pub mod color_stimulus;
pub mod pattern_stimulus;
pub mod patterns;

pub use color_stimulus::ColorStimulus;
pub use pattern_stimulus::PatternStimulus;
pub use patterns::SineGratings;

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
