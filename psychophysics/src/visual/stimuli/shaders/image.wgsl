struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};


@group(0) @binding(1)
var<uniform> transform: mat4x4<f32>;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    // transform the vertex position
    let new_position = transform * vec4<f32>(model.position, 1.);

    return VertexOutput(
        new_position,
        vec2<f32>(model.position.xy),
        model.tex_coords
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.tex_coords);
}
