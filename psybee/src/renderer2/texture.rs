use std::sync::Arc;

use super::helpers::{CacheEntry, Cacheable, Fingerprint};
use image::{DynamicImage, Rgba32FImage, RgbaImage};

#[derive(Debug, Clone)]
pub enum TextureFormat {
    /// 8-bit sRGB with an alpha channel and sRGB encoding.
    Srgba8U,
    /// 8-bit linear with an alpha channel.
    Rgba8U,
    /// 32-bit float linear with an alpha channel.
    Rgba32F,
}

/// A texture.
#[derive(Debug, Clone)]
pub enum Texture {
    /// A texture backed by a RGBA image.
    RgbaImageTexture {
        /// The image data.
        image: Arc<RgbaImage>,
        /// The internal id for caching.
        id: CacheEntry,
        /// The fingerprint. Changes when the image changes.
        fingerprint: u64,
    },
    /// A texture backed by a RGBA image.
    Rgba32FImageTexture {
        /// The image data.
        image: Arc<Rgba32FImage>,
        /// The internal id for caching.
        id: CacheEntry,
        /// The fingerprint. Changes when the image changes.
        fingerprint: u64,
    },
}

impl Texture {
    /// Fram RGBA32F image buffer.
    pub fn from_rgba32f(image: Rgba32FImage) -> Self {
        Self::Rgba32FImageTexture {
            image: Arc::new(image),
            id: CacheEntry::new(),
            fingerprint: rand::random(),
        }
    }
    /// New texture from a RGBA image.
    pub fn from_image(image: DynamicImage, format: TextureFormat) -> Self {
        match format {
            TextureFormat::Srgba8U => {
                let image = image.into_rgba8();
                Self::RgbaImageTexture {
                    image: Arc::new(image),
                    id: CacheEntry::new(),
                    fingerprint: rand::random(),
                }
            }
            TextureFormat::Rgba8U => {
                let image = image.into_rgba8();
                Self::RgbaImageTexture {
                    image: Arc::new(image),
                    id: CacheEntry::new(),
                    fingerprint: rand::random(),
                }
            }
            TextureFormat::Rgba32F => {
                let image = image.into_rgba32f();
                Self::Rgba32FImageTexture {
                    image: Arc::new(image),
                    id: CacheEntry::new(),
                    fingerprint: rand::random(),
                }
            }
        }
    }

    pub fn update_image(&mut self, image: DynamicImage) {
        match self {
            Self::RgbaImageTexture {
                image: old_image,
                fingerprint,
                ..
            } => {
                let new_image = image.into_rgba8();
                if old_image.dimensions() == new_image.dimensions() {
                    *fingerprint = rand::random();
                    *old_image = Arc::new(new_image);
                } else {
                    panic!("Image dimensions do not match");
                }
            }
            _ => panic!("Texture is not backed by an image"),
        }
    }

    /// New texture from a cache entry.
    /// Returns the size of the texture.
    pub fn size(&self) -> (u32, u32) {
        match self {
            Self::RgbaImageTexture { image, .. } => image.dimensions(),
            Self::Rgba32FImageTexture { image, .. } => image.dimensions(),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.width(),
            Self::Rgba32FImageTexture { image, .. } => image.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.height(),
            Self::Rgba32FImageTexture { image, .. } => image.height(),
        }
    }
    /// Returns the image data as a byte slice.
    pub fn data(&self) -> &[u8] {
        match self {
            Self::RgbaImageTexture { image, .. } => &image,
            Self::Rgba32FImageTexture { image, .. } => bytemuck::cast_slice(&image),
        }
    }

    pub fn bytes_per_row(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.width() * 4,
            Self::Rgba32FImageTexture { image, .. } => image.width() * 16,
        }
    }

    pub fn format(&self) -> TextureFormat {
        match self {
            Self::RgbaImageTexture { .. } => TextureFormat::Srgba8U,
            Self::Rgba32FImageTexture { .. } => TextureFormat::Rgba32F,
        }
    }
}

impl Fingerprint for Texture {
    fn fingerprint(&self) -> u64 {
        match self {
            Self::RgbaImageTexture { fingerprint, .. } => *fingerprint,
            Self::Rgba32FImageTexture { fingerprint, .. } => *fingerprint,
        }
    }
}

impl Cacheable for Texture {
    fn cache_id(&self) -> CacheEntry {
        match self {
            Self::RgbaImageTexture { id, .. } => id.clone(),
            Self::Rgba32FImageTexture { id, .. } => id.clone(),
        }
    }
}
