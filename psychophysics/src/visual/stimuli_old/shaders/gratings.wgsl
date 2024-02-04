struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
};

struct GratingStimulusParams {
    phase: f32,
    frequency: f32,
};


@group(0) @binding(0)
var<uniform> params: GratingStimulusParams;
@group(0) @binding(1)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    // transform the vertex position
    let new_position = transform * vec4<f32>(model.position, 1.);

    return VertexOutput(
        new_position,
        vec2<f32>(model.position.xy),
    );

}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos_org = vec4<f32>(in.position_org.xy, 0., 0.);
    var alpha = sin(params.frequency * pos_org.x + params.phase);
    return vec4<f32>(1.0 * alpha, 1.0 * alpha, 1.0 * alpha, 1.0);
}
