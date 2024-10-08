#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniforms {
    pub screen_width: u32,
    pub screen_height: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureUniforms {
    pub size_mode_x: u32,
    pub size_mode_y: u32,
    pub size_value_x: f32,
    pub size_value_y: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PixelFillterUniforms {
    pub filter_type: u32,
    pub filter_params: [f32; 9],
    // be careful with the memory layout of this struct (alignment!)
}

impl PixelFillterUniforms {
    pub fn len() -> usize {
        std::mem::size_of::<Self>()
    }
}
