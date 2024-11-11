struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct ScreenUniforms {
    width: u32,
    height: u32,
};

struct BBox {
    min: vec2<f32>,
    max: vec2<f32>,
};

struct ColourUniforms {
    _transform: mat4x4<f32>,
    _bbox: BBox,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> screen_uniforms: ScreenUniforms;

@group(0) @binding(1)
var<uniform> uniforms: ColourUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return uniforms.color;
}
