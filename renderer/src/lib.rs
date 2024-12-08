pub mod affine;
pub mod brushes;
pub mod colors;
pub mod geoms;
pub mod scenes;
pub mod shapes;
pub mod styles;
pub mod text;
pub mod vello_backend;
pub mod prerenderd_scene;
pub mod effects;

// re-export the image crate
pub use image;

pub type VelloScene = scenes::Scene<vello_backend::VelloBackend>;

pub mod prelude {
    pub use super::affine::*;
    pub use super::brushes::*;
    pub use super::colors::*;
    pub use super::geoms::*;
    pub use super::scenes::*;
    pub use super::shapes::*;
    pub use super::styles::*;
    pub use super::text::*;
    pub use super::VelloScene;
}

pub trait Drawable<Backend> {
    fn draw(&mut self, scene: &mut scenes::Scene<Backend>);
}
