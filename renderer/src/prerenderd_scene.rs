// use crate::affine::Affine;
//
// pub use custom_debug::Debug;
// pub use vello_svg::usvg::Tree;
//
// #[derive(Clone, Debug)]
// pub struct PrerenderedScene {
//     #[debug(skip)]
//     pub scene: vello::Scene,
//     /// The real width of the scene in pixels.
//     pub width: f64,
//     /// The real height of the scene in pixels.
//     pub height: f64,
//     pub transform: Affine,
// }
//
// impl PrerenderedScene {
//     pub fn new(scene: vello::Scene, width: f64, height: f64, transform: Affine) -> Self {
//         Self { scene, width, height, transform }
//     }
//
//     pub fn from_svg_string(svg: &str, transform: Affine) -> Self {
//         let tree = vello_svg::usvg::Tree::from_str(svg, &Default::default()).unwrap();
//         let scene = vello_svg::render_tree(&tree);
//         Self::new(scene, tree.size().width() as f64, tree.size().height() as f64, transform)
//     }
//
//     pub fn set_transform(&mut self, transform: Affine) {
//         self.transform = transform;
//     }
// }
