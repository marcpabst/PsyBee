use crate::GPUState;

use super::{window::WindowState, Window};

pub mod base_stimulus;
pub mod color_stimulus;
pub mod gabor_stimulus;
pub mod image_stimulus;
pub mod pattern_stimulus;
pub mod patterns;
pub mod sprite_stimulus;
pub mod text_stimulus;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
pub mod video_stimulus;

pub use color_stimulus::ColorStimulus;
pub use gabor_stimulus::GaborStimulus;
pub use image_stimulus::ImageStimulus;
pub use pattern_stimulus::PatternStimulus;
pub use sprite_stimulus::SpriteStimulus;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
pub use video_stimulus::VideoStimulus;

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

// macro that implements the Stimulus trait for a newtype with an _inner field that implements the trait
#[macro_export]
macro_rules! impl_stimulus {
    ($newtype:ident, $inner:ty) => {
        use crate::visual::stimuli::Stimulus;
        use crate::GPUState;

        impl Stimulus for $newtype {
            fn prepare(
                &mut self,
                window: &Window,
                window_state: &WindowState,
                gpu_state: &GPUState,
            ) -> () {
                self._inner.prepare(window, window_state, gpu_state);
            }

            fn render(
                &mut self,
                enc: &mut wgpu::CommandEncoder,
                view: &wgpu::TextureView,
            ) -> () {
                self._inner.render(enc, view);
            }
        }
    };
}
