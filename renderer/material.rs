use super::{geometry::Point2D, texture::Texture, uniform_structs};
use std::hash::{Hash, Hasher};

/// An RGBA colour in the current colour space.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Colour {
    /// The red component of the colour.
    pub r: f32,
    /// The green component of the colour.
    pub g: f32,
    /// The blue component of the colour.
    pub b: f32,
    /// The alpha component of the colour.
    pub a: f32,
}

#[rustfmt::skip]
impl Colour {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const DARKGREY: Self = Self { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
    pub const GREY: Self = Self { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const LIGHTGREY: Self = Self { r: 0.8, g: 0.8, b: 0.8, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: Self = Self { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const CYAN: Self = Self { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: Self = Self { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
}

impl Hash for Colour {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.to_bits().hash(state);
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

/// The type of gradient.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientType {
    /// A linear gradient.
    Linear,
    /// A radial gradient.
    Radial,
    /// A conic gradient.
    Conic,
}

/// The extend of a gradient.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientExtent {
    /// Exact extent. Interpretation depends on the gradient type. Use in combination with
    /// `GradientRepeatMode' to control the repeat behaviour.
    /// - For linear gradients, this is the total length of the gradient in pixels.
    /// - For radial gradients, this is the total radius of the gradient in pixels.
    /// - For conic gradients, this is the total angle of the gradient in degrees.
    Absolute(f32),
    /// Relative extent. Interpretation depends on the gradient type. Use in combination with
    /// `GradientRepeatMode' to control the repeat behaviour.
    /// - For linear gradients, this is the total length of the gradient as a fraction of the shape's
    /// bounding box.
    /// - For radial gradients, this is the total radius of the gradient as a fraction of the shape's
    /// bounding box.
    /// - For conic gradients, this is the total angle as a fraction of full circle.
    Relative(f32),
    /// Choose the extent that fills the available space. Interpretation depends on the gradient type.
    /// - For linear gradients, this will stretch the gradient to fill the shape (or the shape's bounding box).
    /// - For radial gradients, this will stretch the gradient to fill the shape (or the shape's bounding box).
    /// for a conic gradient, this is
    /// identical to `Exact(360.0)`).
    Fill,
}

/// The repeat mode of a gradient.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientRepeatMode {
    /// Clamp the gradient, repeating the last colour.
    Clamp,
    /// Repeat the gradient.
    Repeat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureSize {
    /// Use the original texture size.
    Original,
    /// Use the an exact texture size in pixels.
    Absolute(f32),
    /// Use a texture size relative to the shape's bounding box.
    Relative(f32),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureRepeat {
    /// Clamp the texture, repeating the last pixel.
    Clamp,
    /// Repeat the texture.
    Repeat,
    /// Repeat the texture, mirroring every other repeat.
    Mirror,
    /// Do not repeat the texture. This is not supported for all backends.
    None,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureFilter {
    /// Nearest neighbour filtering.
    Nearest,
    /// Linear filtering.
    Linear,
}

impl TextureSize {
    pub fn get(&self) -> (u32, f32) {
        match self {
            TextureSize::Original => (0, 0.0),
            TextureSize::Absolute(size) => (1, *size),
            TextureSize::Relative(size) => (2, *size),
        }
    }
}

#[derive(Clone)]
pub struct TextureMaterial {
    pub texture: Texture,
    pub size_x: TextureSize,
    pub size_y: TextureSize,
    pub repeat_x: TextureRepeat,
    pub repeat_y: TextureRepeat,
    pub filter: TextureFilter,
}

#[derive(Clone)]
pub struct GradientMaterial {
    /// The type of gradient.
    pub gradient_type: GradientType,
    /// Extent of the gradient.
    pub extent: GradientExtent,
    /// The repeat mode of the gradient.
    pub repeat: GradientRepeatMode,
    /// The position of the gradient centre, relative to the centre of the shape's bounding box.
    pub centre: Point2D,
    /// The colours of the gradient. All colours are equally spaced, and no interpolation is done.
    /// This means that you will need to pre-calculate the colours if you want a smooth gradient.
    pub ramp_texture: Texture,
    /// The rotation of the gradient in degrees.
    pub rotation: f32,
}

/// A material that defines how a shape should be rendered.
#[derive(Clone)]
pub enum Material {
    /// All pixels are the same colour.
    Colour(Colour),
    /// Apply the given texture to the shape.
    Texture(TextureMaterial),
    /// Apply the given gradient to the shape.
    Gradient(GradientMaterial),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Color,
    Texture,
    Gradient,
}

impl Material {
    /// Returns the vertex shader module for this material.
    pub fn vertex_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::include_wgsl!("shaders/vertex.wgsl"))
    }

    /// Returns the fragment shader module for this material.
    pub fn fragment_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        match self {
            Self::Colour(..) => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/colour.wgsl"))
            }
            Self::Texture(..) => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/texture.wgsl"))
            }
            Self::Gradient(..) => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/gradient.wgsl"))
            }
        }
    }

    /// Returns the material type.
    pub fn material_type(&self) -> MaterialType {
        match self {
            Self::Colour { .. } => MaterialType::Color,
            Self::Texture { .. } => MaterialType::Texture,
            Self::Gradient { .. } => MaterialType::Gradient,
        }
    }

    /// Returns the texture for this material, if it has one.
    pub fn texture(&self) -> Option<&Texture> {
        match self {
            Self::Texture(TextureMaterial { texture, .. }) => Some(texture),
            Self::Gradient(GradientMaterial { ramp_texture, .. }) => Some(ramp_texture),
            _ => None,
        }
    }

    /// Returns true if the material has a texture.
    pub fn has_texture(&self) -> bool {
        self.texture().is_some()
    }

    /// Returns the uniform buffer for this material.
    pub fn uniform_bytes(&self) -> Vec<u8> {
        match self {
            Self::Colour(colour) => bytemuck::bytes_of(colour).to_vec(),
            Self::Texture(TextureMaterial { size_x, size_y, .. }) => {
                let uniforms = uniform_structs::TextureUniforms {
                    size_mode_x: size_x.get().0,
                    size_mode_y: size_y.get().0,
                    size_value_x: size_x.get().1,
                    size_value_y: size_y.get().1,
                };

                bytemuck::bytes_of(&uniforms).to_vec()
            }
            Self::Gradient(GradientMaterial {
                centre, rotation, ..
            }) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(bytemuck::bytes_of(centre));
                bytes.extend_from_slice(bytemuck::bytes_of(rotation));
                bytes
            }
        }
    }

    /// Texture repeat modes.
    pub fn texture_repeat_modes(&self) -> Option<(TextureRepeat, TextureRepeat)> {
        match self {
            Self::Texture(TextureMaterial {
                repeat_x, repeat_y, ..
            }) => Some((*repeat_x, *repeat_y)),
            _ => None,
        }
    }

    pub fn texture_filter(&self) -> Option<TextureFilter> {
        match self {
            Self::Texture(TextureMaterial { filter, .. }) => Some(*filter),
            _ => None,
        }
    }

    /// Returns the size of the uniform buffer for this material.
    pub fn uniform_buffer_size(&self) -> usize {
        self.material_type().uniform_buffer_size()
    }
}

impl MaterialType {
    /// Returns the name of the material type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Texture => "Texture",
            Self::Gradient => "Gradient",
        }
    }

    /// Returns the size of the uniform buffer for this material.
    pub fn uniform_buffer_size(&self) -> usize {
        match self {
            Self::Color { .. } => std::mem::size_of::<[f32; 4]>(),
            Self::Texture { .. } => std::mem::size_of::<[f32; 4]>(),
            Self::Gradient { .. } => std::mem::size_of::<[f32; 4]>(),
        }
    }

    /// Returns true if the material has a texture.
    pub fn has_texture(&self) -> bool {
        match self {
            Self::Texture => true,
            Self::Gradient => true,
            _ => false,
        }
    }
}
