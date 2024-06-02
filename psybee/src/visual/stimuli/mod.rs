use uuid::Uuid;

use super::geometry::Size;
use super::window::InternalWindowState;
use super::Window;
use crate::GPUState;

pub mod base_stimulus;
pub mod color_stimulus;
pub mod gabor_stimulus;
pub mod image_stimulus;
pub mod pattern_stimulus;
pub mod patterns;
pub mod sprite_stimulus;


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
pub trait Stimulus: Send + Sync + downcast_rs::Downcast + dyn_clone::DynClone {
    /// Prepare the renderable object for rendering.
    fn prepare(&mut self, window: &Window, window_state: &InternalWindowState, gpu_state: &GPUState) -> ();
    /// Render the object to the screen.
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
    /// Check if the stimulus contains a specific Point.
    fn contains(&self, x: Size, y: Size) -> bool;
    /// Return the UUID that identifies the stimulus.
    fn uuid(&self) -> Uuid;
    /// Check if two stimuli are equal.
    fn equal(&self, other: &dyn Stimulus) -> bool {
        self.uuid() == other.uuid()
    }
    /// Returns true if the stimulus is currently visible.
    fn visible(&self) -> bool {
        true
    }
    /// Set the visibility of the stimulus.
    fn set_visible(&self, _is_visible: bool) {
        // do nothing by default
    }

    /// Hide the stimulus. This is a convenience method that calls
    /// `set_visible(false)`.
    fn hide(&self) -> () {
        self.set_visible(false);
    }

    /// Show the stimulus. This is a convenience method that calls
    /// `set_visible(true)`.
    fn show(&self) -> () {
        self.set_visible(true);
    }

    /// Toggle the visibility of the stimulus.
    fn toggle_visibility(&self) -> () {
        self.set_visible(!self.visible());
    }
}
downcast_rs::impl_downcast!(Stimulus);

// macro that implements the Stimulus trait for a newtype with an _inner field
// that implements the trait
#[macro_export]
macro_rules! impl_stimulus {
    ($newtype:ident, $inner:ty) => {
        use uuid::Uuid;

        use crate::visual::geometry::Size;
        use crate::visual::stimuli::Stimulus;
        use crate::GPUState;

        impl Stimulus for $newtype {
            fn prepare(&mut self, window: &Window, window_state: &InternalWindowState, gpu_state: &GPUState) -> () {
                self._inner.prepare(window, window_state, gpu_state);
            }

            fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
                self._inner.render(enc, view);
            }

            fn contains(&self, x: Size, y: Size) -> bool {
                self._inner.contains(x, y)
            }

            fn uuid(&self) -> Uuid {
                self._inner.uuid()
            }
        }
    };
}
