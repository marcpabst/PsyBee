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

struct PixelFillterUniforms {
    filter_type: u32,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
    g: f32,
}


@group(0) @binding(0)
var<uniform> screen_uniforms: ScreenUniforms;

@group(0) @binding(1)
var<uniform> uniforms: ColourUniforms;

@group(0) @binding(2)
var<uniform> pixel_filter_uniforms: PixelFillterUniforms;

fn apply_pixel_filter(loc: vec2<f32>, value: vec4<f32>) -> vec4<f32> {
    if pixel_filter_uniforms.filter_type == 1 // Gaussian
    {
        // the centre
        //let centre = vec2<f32>(pixel_filter_uniforms.a, pixel_filter_uniforms.b);
        let centre = vec2<f32>(0.5, 0.5);
        // the standard deviation
        //let sigma = vec2<f32>(pixel_filter_uniforms.c, pixel_filter_uniforms.d);
        let sigma = vec2<f32>(60, 60);

        // apply the Gaussian envelope
        let diff = loc - centre;
        let dist = dot(diff, diff);
        let factor = exp(-dist / (2.0 * sigma.x * sigma.x));
        return vec4<f32>(value.xyz, value.w * factor);
    }

    return value; // no filter
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let out = uniforms.color;
    return apply_pixel_filter(in.position_org, out);
}
