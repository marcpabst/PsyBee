struct VertexOutput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct Uniforms {
    // the direction of the linear gradient
    rotation: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// this is a 1x512 ramp texture
@group(0) @binding(2)
var texture: texture_2d<f32>;

@group(0) @binding(3)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // rotate the gradient
    let rotation = uniforms.rotation;
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);

    // compute the texture coordinate
    let tex_coords = in.tex_coords;
    let x = tex_coords.x * cos_r - tex_coords.y * sin_r;
    let y = tex_coords.x * sin_r + tex_coords.y * cos_r;

    // sample the texture
    return textureSample(texture, texture_sampler, vec2<f32>(x, y));
}


