use naga;
use naga_oil;

fn main() {
    let wgsl_str1 = "
    struct UniformColor {
        color: vec4<f32>,
    }

    struct VertexOutput {
        @location(0) position_org: vec4<f32>,
    };

    @group(1) @binding(0)
    var<uniform> uniform_color: UniformColor;

    fn some_fn(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {
        let params = uniform_color;
        return vec4<f32>(params.color.r, params.color.g, params.color.b, params.color.a);
    }

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let x = in.position_org.x;
        let y = in.position_org.y;
        let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        return some_fn(x, y, color);
    }
    ";

    let wgsl_str2 = "
    struct UniformColor {
        color: vec4<f32>,
    }

    @group(1) @binding(0)
    var<uniform> uniform_color: UniformColor;

    fn some_fn(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {
        let params = uniform_color;
        return vec4<f32>(params.color.r, params.color.g, params.color.b, params.color.a);
    }

    fn main(in: VertexOutput) -> @location(0) vec4<f32> {
        let x = in.position_org.x;
        let y = in.position_org.y;
        let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        return some_fn(x, y, color);
    }
    ";

    let mut composer = naga_oil::compose::Composer::default();

    // use the naga_oil crate to combine the two modules
    let mut composer = naga_oil::compose::Composer::default();

    let mut load_composable = |source: &str, file_path: &str| {
        match composer.add_composable_module(
            naga_oil::compose::ComposableModuleDescriptor {
                source,
                file_path,
                ..Default::default()
            },
        ) {
            Ok(_module) => {
                // println!("{} -> {:#?}", module.name, module)
            }
            Err(e) => {
                println!("? -> {e:#?}")
            }
        }
    };

    load_composable(wgsl_str1, "file1.wgsl");
    load_composable(wgsl_str2, "file2.wgsl");
}
