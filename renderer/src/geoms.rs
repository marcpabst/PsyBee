use super::affine::Affine;
use super::brushes::{Brush, Image};
pub use super::scenes::Scene;
use super::shapes::{Point, Rectangle, Shape};
use super::styles::{FillStyle, ImageFitMode, Style};

// A geometric object that can be rendered, consisting of a shape and a brush.
#[derive(Debug, Clone)]
pub struct Geom<S: Shape> {
    pub style: Style,
    pub shape: S,
    pub brush: Brush,
    pub transform: Affine,
    pub brush_transform: Option<Affine>,
}

pub trait GeomTrait {
    fn new_image(
        image: Image, // the image to render
        x: f64, // top left x coordinate of the image geom
        y: f64, // top left y coordinate of the image geom
        width: f64, // width of the image geom
        height: f64, // height of the image geom
        transform: Affine, // transformation of the image geom
        image_x: f64, // x offset of the image
        image_y: f64, // y offset of the image
        fit_mode: ImageFitMode, // how to fit the image
        edge_mode: crate::brushes::Extend,  // how to handle edges
    ) -> Geom<Rectangle> {
        let shape = Rectangle {
            a: Point {
                x: x - width / 2.0,
                y: y - height / 2.0,
            },
            b: Point {
                x: x + width / 2.0,
                y: y + height / 2.0,
            },
        };


        let org_width = image.width as f64;
        let org_height = image.height as f64;

        let brush = Brush::Image{
            image,
            x: image_x,
            y: image_y,
            fit_mode,
            edge_mode,
        };

        let brush_transform = match fit_mode {
            ImageFitMode::Original => None,
            ImageFitMode::Fill => Some(Affine::scale_xy(width / org_width, height / org_height)
                * Affine::translate(x - width / 2.0, y - height / 2.0)),
            ImageFitMode::Exact { width: new_width, height: new_height } => {
                Some(Affine::scale_xy(new_width / org_width, new_height / org_height))
            }
        };

        // Center the brush.
        // let brush_transform =
        //     brush_transform.map(|t| t * Affine::translate(x - width / 2.0, y - height / 2.0));

        // Apple image_x and image_y.
        let brush_transform = brush_transform.map(|t| t * Affine::translate(image_x, image_y));

        Geom {
            style: Style::Fill(FillStyle::NonZero),
            shape,
            brush,
            transform,
            brush_transform,
        }
    }
}

impl GeomTrait for Geom<Rectangle> {}
