pub mod affine;
pub mod bitmaps;
pub mod brushes;
pub mod colors;
pub mod effects;
pub mod prerenderd_scene;
mod renderer;
pub mod scenes;
pub mod shapes;
mod skia_backend;
pub mod styles;
pub mod text;
mod utils;
pub mod vello_backend;
// mod skia_backend;

// re-export the image crate
pub use image;

// re-export wgpu crate
pub use wgpu;

// re-export the renderer
pub use renderer::DynamicRenderer;

pub enum Backend {
    Vello,
    Skia,
}

pub mod prelude {
    pub use super::affine::*;
    pub use super::brushes::*;
    pub use super::colors::*;
    pub use super::scenes::*;
    pub use super::shapes::*;
    pub use super::styles::*;
    pub use super::text::*;
}
