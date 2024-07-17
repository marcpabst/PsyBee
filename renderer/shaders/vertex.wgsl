struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct ScreenUniforms {
    width: u32,
    height: u32,
};

@group(0) @binding(0)
var<uniform> screen_uniforms: ScreenUniforms;

@group(0) @binding(1)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    // transform the vertex position to clip space (from pixel space), using the screen size
    var new_position = vec2<f32>(
        (in.position.x / f32(screen_uniforms.width)) * -2.0,
        (in.position.y / f32(screen_uniforms.height)) * -2.0
    );

    // transform the vertex position
    let transform3x3 = mat3x3<f32>(transform[0].xyz, transform[1].xyz, transform[2].xyz);
    new_position = (transform3x3 * vec3(new_position, 1.0)).xy;

    return VertexOutput(
        vec4<f32>(new_position.xy, 0.0, 1.0),
        vec2<f32>(in.position.xy),
        vec2<f32>(new_position.xy),
    );
}
