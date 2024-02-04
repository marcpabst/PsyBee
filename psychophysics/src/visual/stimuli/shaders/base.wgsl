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


@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;
@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    // transform the vertex position
    let transform3x3 = mat3x3<f32>(transform[0].xyz, transform[1].xyz, transform[2].xyz);
    let new_position = transform3x3 * vec3(model.position.xy, 1.0);
    let new_position2 = vec4(new_position, 1.0);

    return VertexOutput(
        new_position2,
        vec2<f32>(model.position.xy),
        model.tex_coords
    );
}