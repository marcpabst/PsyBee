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
    /// A texture backed by a raw buffer.
    RawTexture {
        /// The raw buffer.
        buffer: Vec<u8>,
        /// The width of the texture.
        width: u32,
        /// The height of the texture.
        height: u32,
        /// The internal id for caching.
        id: CacheEntry,
        /// The fingerprint. Changes when the buffer changes.
        fingerprint: u64,
        /// The format of the texture.
        format: TextureFormat,
    },
}

impl Texture {
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

    /// New texture from a raw buffer.
    pub fn from_raw(buffer: Vec<u8>, width: u32, height: u32, format: TextureFormat) -> Self {
        Self::RawTexture {
            buffer,
            width,
            height,
            id: CacheEntry::new(),
            fingerprint: rand::random(),
            format,
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
            Self::RawTexture { width, height, .. } => (*width, *height),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.width(),
            Self::Rgba32FImageTexture { image, .. } => image.width(),
            Self::RawTexture { width, .. } => *width,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.height(),
            Self::Rgba32FImageTexture { image, .. } => image.height(),
            Self::RawTexture { height, .. } => *height,
        }
    }
    /// Returns the image data as a byte slice.
    pub fn data(&self) -> &[u8] {
        match self {
            Self::RgbaImageTexture { image, .. } => &image,
            Self::Rgba32FImageTexture { image, .. } => bytemuck::cast_slice(&image),
            Self::RawTexture { buffer, .. } => buffer,
        }
    }

    pub fn bytes_per_row(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.width() * 4,
            Self::Rgba32FImageTexture { image, .. } => image.width() * 16,
            Self::RawTexture { width, .. } => *width * 4,
        }
    }

    pub fn format(&self) -> TextureFormat {
        match self {
            Self::RgbaImageTexture { .. } => TextureFormat::Srgba8U,
            Self::Rgba32FImageTexture { .. } => TextureFormat::Rgba32F,
            Self::RawTexture { .. } => TextureFormat::Rgba8U,
        }
    }
}

impl Fingerprint for Texture {
    fn fingerprint(&self) -> u64 {
        match self {
            Self::RgbaImageTexture { fingerprint, .. } => *fingerprint,
            Self::Rgba32FImageTexture { fingerprint, .. } => *fingerprint,
            Self::RawTexture { fingerprint, .. } => *fingerprint,
        }
    }
}

impl Cacheable for Texture {
    fn cache_id(&self) -> CacheEntry {
        match self {
            Self::RgbaImageTexture { id, .. } => id.clone(),
            Self::Rgba32FImageTexture { id, .. } => id.clone(),
            Self::RawTexture { id, .. } => id.clone(),
        }
    }
}
