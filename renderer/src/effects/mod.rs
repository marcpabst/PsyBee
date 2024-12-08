// post-processing

pub trait EffectShader {
    /// Returns the WebGPU compute shader code for the effect.
    fn wgsl(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct GrayscaleEffectShader;

impl EffectShader for GrayscaleEffectShader {
    fn wgsl(&self) -> String {
        r#"
            [[block]]
            struct Uniforms {
                texture: texture_2d<f32>;
            };

            [[group(0), binding(0)]]
            var<uniform> uniforms: Uniforms;

            [[group(0), binding(1)]]
            var output: texture_2d<f32>;

            [[stage(compute), workgroup_size(1)]]
            fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
                let color: vec4<f32> = uniforms.texture.read(gid.xy);
                let gray: f32 = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
                output.write(gid.xy, vec4<f32>(gray, gray, gray, color.a));
            }
        "#
        .to_string()
    }
}
