struct Params {
    r: P,
    g: P,
    b: P,
    correction: u32, // 0: none, 1: psychopy, 2: polylog4, 3: polylog5, 4: polylog6
};

struct P {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
    g: f32,
    h: f32,
};

fn npow(x: f32, n: f32) -> f32 {
    return sign(x) * pow(abs(x), n);
}

fn pure_gamma_inv_eotf(value: f32, params: P) -> f32 {
    return npow(value, params.c);
}

fn psychopy_scaled_inv_eotf(value: f32, params: P) -> f32 {
    return (npow(( (1.0 - value) * npow(params.a, params.c) + value * npow((params.a + params[2]), params.c)), (1/params.c)) - params.a) / params[2];
}

fn polylog4(x: f32, params: P) -> f32 {
    let logx = log(x);
    return params.a + params.b * logx + params.c * npow(logx, 2.0) + params.d * npow(logx, 3.0) + params.e * npow(logx, 4.0);
}

fn polylog4_horner(x: f32, params: P) -> f32 {
    // use Horner's method to evaluate the polynomial
    let logx = log(x);
    return params.a + logx * (params.b + logx * (params.c + logx * (params.d + logx * params.e)));
}

fn polylog5(x: f32, params: P) -> f32 {
    let logx = log(x);
    let out = params.a + params.b * logx + params.c * npow(logx, 2.0) + params.d * npow(logx, 3.0) + params.e * npow(logx, 4.0) + params.f * npow(logx, 5.0);
    return out;
}

fn polylog5_horner(x: f32, params: P) -> f32 {
    // use Horner's method to evaluate the polynomial
    let logx = log(x);
    return params.a + logx * (params.b + logx * (params.c + logx * (params.d + logx * (params.e + logx * params.f))));
}

fn polylog6(x: f32, params: P) -> f32 {
    let logx = log(x);
    return params.a + params.b * logx + params.c * npow(logx, 2.0) + params.d * npow(logx, 3.0) + params.e * npow(logx, 4.0) + params.f * npow(logx, 5.0) + params.g * npow(logx, 6.0);
}

@vertex
fn vs_main(@builtin(vertex_index) ix: u32) -> @builtin(position) vec4<f32> {
    // Generate a full screen quad in normalized device coordinates
    var vertex = vec2(-1.0, 1.0);
    switch ix {
        case 1u: {
            vertex = vec2(-1.0, -1.0);
        }
        case 2u, 4u: {
            vertex = vec2(1.0, -1.0);
        }
        case 5u: {
            vertex = vec2(1.0, 1.0);
        }
        default: {}
    }
    return vec4(vertex, 0.0, 1.0);
}

// bind the input texture to the shader
@group(0) @binding(0)
var fine_output: texture_2d<f32>;

// bind the uniform buffer to the shader
@group(0) @binding(1)
var<uniform> params: Params;


@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let rgba_sep = textureLoad(fine_output, vec2<i32>(pos.xy), 0);
    let rgb_pm = vec3(rgba_sep.rgb * rgba_sep.a);

    if params.correction == 0 {
        let rgb = vec3(
            rgb_pm.r,
            rgb_pm.g,
            rgb_pm.b
        );
        return vec4(rgb, rgba_sep.a);
    }
    else if params.correction == 1 {
        let rgb = vec3(
            psychopy_scaled_inv_eotf(rgb_pm.r, params.r),
            psychopy_scaled_inv_eotf(rgb_pm.g, params.g),
            psychopy_scaled_inv_eotf(rgb_pm.b, params.b)
        );
        return vec4(rgb, rgba_sep.a);
    }
    else if params.correction == 2 {
        let rgb = vec3(
            polylog4(rgb_pm.r, params.b),
            polylog4(rgb_pm.g, params.g),
            polylog4(rgb_pm.b, params.b)
        );
        return vec4(rgb, rgba_sep.a);
    }
    else if params.correction == 3 {
        let rgb = vec3(
            polylog5_horner(rgb_pm.r, params.b),
            polylog5_horner(rgb_pm.g, params.g),
            polylog5_horner(rgb_pm.b, params.b)
        );
        return vec4(rgb, rgba_sep.a);
    }
    else if params.correction == 4 {
        let rgb = vec3(
            polylog6(rgb_pm.r, params.b),
            polylog6(rgb_pm.g, params.g),
            polylog6(rgb_pm.b, params.b)
        );
        return vec4(rgb, rgba_sep.a);
    }


    return vec4(rgb_pm, rgba_sep.a);
}