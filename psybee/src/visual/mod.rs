//! This module contains various visual stimuli that can be used in
//! psybee experiments.
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
