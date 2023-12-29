use async_lock::{Mutex, MutexGuard};
use futures_lite::future::block_on;
use futures_lite::FutureExt;
use image;

pub use web_time as time;

pub trait BlockingLock<T: ?Sized> {
    fn lock_blocking(&self) -> MutexGuard<'_, T>;
}

impl<T: ?Sized> BlockingLock<T> for Mutex<T> {
    fn lock_blocking(&self) -> MutexGuard<'_, T> {
        block_on(self.lock())
    }
}

/// Includes an image as a reference to a byte array.
/// This is useful for including images in the binary.
/// The image is loaded at compile time.
///
/// # Example
///
/// ```
/// let image = include_image!("image.png");
/// ```
#[macro_export]
macro_rules! include_image {
    ($name:literal) => {{
        let bytes = include_bytes!($name);
        image::load_from_memory(bytes).unwrap()
    }};
}
