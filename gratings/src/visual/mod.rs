pub mod gratings;
pub mod shape;
pub mod text;





use wgpu::{
    CommandEncoder, Device, Queue, RenderPass,
    SurfaceConfiguration,
};


// Renderable trait should be implemented by all visual stimuli
// the API is extremely simple: render() and update() and follows the
// the middlewares pattern used by wgpu
pub trait Renderable {
    fn render<'pass>(&'pass self, device: &mut Device, pass: &mut RenderPass<'pass>) -> ();
    fn update(
        &mut self,
        device: &mut Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        config: &SurfaceConfiguration,
    ) -> ();
}
