// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Pattern stimuli for visual stimuli.

pub mod checkerboard;
pub mod gabor;
pub mod gabor_patch;
pub mod image;
pub mod sprite;
pub mod uniform;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
pub mod video;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
pub mod camera;

pub use checkerboard::Checkerboard;
pub use gabor::Gabor;
pub use gabor_patch::GaborPatch;
pub use image::Image;
pub use sprite::Sprite;
pub use uniform::Uniform;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios")))]
pub use video::Video;
