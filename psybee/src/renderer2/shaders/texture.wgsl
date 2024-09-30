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

struct TextureUniforms {
    _transform: mat4x4<f32>,
    bbox: BBox,
    size_mode_x: u32, // 0: original, 1: absolute, 2: relative
    size_mode_y: u32, // 0: original, 1: absolute, 2: relative
    size_value_x: f32,
    size_value_y: f32,
    repeat_mode_x: u32, // 0: clamp, 1: repeat, 2: mirror
    repeat_mode_y: u32, // 0: clamp, 1: repeat, 2: mirror
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
    h: f32,
    i: f32,
}

@group(0) @binding(0)
var<uniform> screen_uniforms: ScreenUniforms;

@group(0) @binding(1)
var<uniform> uniforms: TextureUniforms;

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var<uniform> pixel_filter_uniforms: PixelFillterUniforms;

fn apply_pixel_filter(loc: vec2<f32>, value: vec4<f32>) -> vec4<f32> {
    if pixel_filter_uniforms.filter_type == 3 // Gaussian
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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_dims = textureDimensions(texture);

    var tex_coords_x = 0.0;
    // calculate x texture coordinate
    if (uniforms.size_mode_x == 0u) {
        // original size
        tex_coords_x = (in.position_org.x - uniforms.bbox.min.x) / (uniforms.bbox.max.x - uniforms.bbox.min.x);
    } else if (uniforms.size_mode_x == 1u) {
        // absolute size in pixels
        tex_coords_x = (in.position_org.x - uniforms.bbox.min.x) / uniforms.size_value_x;
    } else if (uniforms.size_mode_x == 2u) {
        // relative size fractions of the bounding box width
        tex_coords_x = (1/uniforms.size_value_x) * (in.position_org.x - uniforms.bbox.min.x) / (uniforms.bbox.max.x - uniforms.bbox.min.x);
    } else {
        // return red if mode is invalid
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    var tex_coords_y = 0.0;
    // calculate y texture coordinate
    if (uniforms.size_mode_y == 0u) {
        // original size
        tex_coords_y = (in.position_org.y - uniforms.bbox.min.y) / (uniforms.bbox.max.y - uniforms.bbox.min.y);
    } else if (uniforms.size_mode_y == 1u) {
        // absolute size in pixels
        tex_coords_y = (in.position_org.y - uniforms.bbox.min.y) / uniforms.size_value_y;

    } else if (uniforms.size_mode_y == 2u) {
        // relative size fractions of the bounding box height
        tex_coords_y = (1/uniforms.size_value_y) * (in.position_org.y - uniforms.bbox.min.y) / (uniforms.bbox.max.y - uniforms.bbox.min.y);
    } else {
        // return red if mode is invalid
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    // sample the texture
    var out = textureSample(texture, texture_sampler, vec2<f32>(tex_coords_x, tex_coords_y));
    return apply_pixel_filter(in.position_org, out);
}

// // if exact mode, we don't need to do anything
// if (uniforms.mode == 0u) {
//     let tex_coords = (in.position_org.xy - uniforms.bbox.min) / vec2<f32>(f32(tex_dims.x), f32(tex_dims.y));
//     return textureSample(texture, texture_sampler, tex_coords);
// } else if (uniforms.mode == 1u) {
//     let bbox_center = (uniforms.bbox.min + uniforms.bbox.max) / 2.0;
//     let offset = in.position_org.xy - bbox_center - vec2<f32>(f32(tex_dims.x) / 2.0, f32(tex_dims.y) / 2.0);
//     let tex_coords = (in.position_org.xy - uniforms.bbox.min + offset) / vec2<f32>(f32(tex_dims.x), f32(tex_dims.y));
//     return textureSample(texture, texture_sampler, tex_coords);
// } else if (uniforms.mode == 2u) {

//     // if stretch mode, we need to normalize the coordinates to [0, 1] based on the bounding box
//     let tex_coords = (in.position_org - uniforms.bbox.min) / (uniforms.bbox.max - uniforms.bbox.min);

//     return textureSample(texture, texture_sampler, tex_coords);
// }
