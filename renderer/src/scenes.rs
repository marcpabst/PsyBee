use super::affine::Affine;
use super::colors;

use super::styles::CompositeMode;
use super::styles::MixMode;

// A Scene that can be rendered onto.
#[derive(Debug, Clone)]
pub struct Scene<Backend> {
    /// The background color of the scene.
    pub background_color: colors::RGBA,
    pub width: u32,
    pub height: u32,
    // Backend specifics data.
    pub backend: Backend,
}

pub trait SceneTrait<Backend, ClipShape: Clone> {
    fn scene_mut(&mut self) -> &mut Scene<Backend>;
    fn scene(&self) -> &Scene<Backend>;
    fn start_layer(
        &mut self,
        mix_mode: MixMode,
        composite_mode: CompositeMode,
        clip: ClipShape,
        clip_transform: Affine,
        layer_transform: Option<Affine>,
        alpha: f32,
    );
    fn end_layer(&mut self);
    fn draw_alpha_mask(
        &mut self,
        mask: impl FnOnce(&mut Scene<Backend>),
        item: impl FnOnce(&mut Scene<Backend>),
        clip: ClipShape,
        clip_transform: Affine,
    ) {
        self.start_layer(
            MixMode::Normal,
            CompositeMode::SourceOver,
            clip.clone(),
            clip_transform,
            None,
            1.0,
        );
        mask(self.scene_mut());

        self.start_layer(
            MixMode::Multiply,
            CompositeMode::SourceIn,
            clip.clone(),
            clip_transform,
            None,
            1.0,
        );

        item(self.scene_mut());

        self.end_layer();
        self.end_layer();
    }
}
