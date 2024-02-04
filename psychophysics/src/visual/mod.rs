//! This module contains various visual stimuli that can be used in
//! psychophysics experiments.
//!
//! The stimuli are implemented as
//! [renderable](trait.Renderable.html) objects that can be added to a
//! [frame](struct.Frame.html) and rendered to the screen.
//!
//! # Background
//!
//! The visual stimuli are implemented using the [wgpu](https://wgpu.rs/)
//! library. This library is a low-level graphics library that allows
//! for fast rendering of 2D and 3D graphics. The library is based on
//! the [WebGPU](https://gpuweb.github.io/gpuweb/) standard and is
//! supported on most platforms. The library is still in development
//! and is not yet stable. This means that the API might change in the
//! future.
//!
//! # Stimuli
//!
//! The following stimuli are currently implemented:
//!
//! * [Image](struct.ImageStimulus.html)
//! * [Gratings](struct.GratingsStimulus.html)
//! * [Text](struct.TextStimulus.html)
//!
//! # Example
//!
//! The following example shows how to create a simple experiment with
//! a fixation cross and a grating stimulus.
pub mod color;
pub mod geometry;
// pub mod stimuli;
pub mod stimuli;
pub mod window;

pub use window::Window;

use async_trait::async_trait;
use wgpu::{Device, Queue, SurfaceConfiguration};

/// Trait for all renderable objects. This mostly follows the `wgpu`
/// recommendations for rendering middleware. This is trait is mostly
/// used internall but can be used to create custom stimuli.
#[async_trait(?Send)]
pub trait Renderable {
    /// Prepare the renderable object for rendering. By default this
    /// function calls `prepare_async` function in a blocking manner.
    async fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &window::Window,
    ) -> ();
    /// Render the object to the screen.
    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> ();
    fn is_finnished(&self) -> bool {
        false
    }
}
