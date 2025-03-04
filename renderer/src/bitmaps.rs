use std::{any::Any, fmt::Debug};

use image::DynamicImage;

pub use super::scenes::Scene;

#[derive(Debug)]
pub struct DynamicBitmap(pub Box<dyn Bitmap>);

impl DynamicBitmap {
    pub fn try_as<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.0.as_any().downcast_ref::<T>()
    }
}

pub trait Bitmap: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
