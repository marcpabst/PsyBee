use ndarray::{s, Array, Axis};
use std::collections::HashMap;
use ort::{CPUExecutionProvider, Session};
use tract_onnx::prelude::*;
use web_time::SystemTime;
use wonnx::{
    utils::{attribute, graph, initializer, model, node, tensor, OutputTensor},
    SessionError, WonnxError,
};

async fn run_wonnx(onnx_file: &[u8], n: usize) {
    let session = wonnx::Session::from_bytes(onnx_file).await.unwrap();

    let mut times = vec![1.0f32; n];

    for i in 0..n {
        // create input tensor (256x256x3, 1.0f32)
        let input: Array::<f32, ndarray::Dim<[usize; 4]>> = Array::zeros((1, 256, 256, 3));

        let mut input_data = HashMap::new();
        input_data.insert("input_12".to_string(), input.as_slice().unwrap().into());

        let start = SystemTime::now();
        let outputs = session.run(&input_data).await.unwrap();
        let end = SystemTime::now();

        // add duration to times
        let duration = end.duration_since(start).unwrap().as_millis() as f32;
        times[i] = duration;
    }

    println!(
        "WONNX: Average inference time: {} ms",
        times.iter().sum::<f32>() / n as f32
    );
}

fn main() {
    // TRACT
    // load onnx model at compile time
    let onnx_file = include_bytes!("modified_face_landmarks_detector2-fixed.onnx");
    let onnx_file_sim = include_bytes!("modified_face_landmarks_detector2-fixed.onnx");
    let mut onnx_reader = std::io::Cursor::new(onnx_file);

    let model = tract_onnx::onnx()
        .model_for_read(&mut onnx_reader)
        .unwrap()
        .with_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 256, 256, 3)),
        )
        .unwrap()
        // optimize the model
        .into_optimized()
        .unwrap()
        // make the model runnable and fix its inputs and outputs
        .into_runnable()
        .unwrap();

    let n = 10;

    let mut times = vec![1.0f32; n];

    for i in 0..n {
        // create input tensor (256x256x3, 1.0f32)
        let dummy_input = Tensor::zero_dt(DatumType::F32, &[1, 256, 256, 3]).unwrap();

        let start = SystemTime::now();
        let _outputs = model.run(tvec![dummy_input.into()]).unwrap();
        let end = SystemTime::now();

        // add duration to times
        let duration = end.duration_since(start).unwrap().as_millis() as f32;
        times[i] = duration;
    }

    println!(
        "TRACT: Average inference time: {} ms",
        times.iter().sum::<f32>() / n as f32
    );

    // ORT
    ort::init()
        .with_name("Model")
        .with_execution_providers([CPUExecutionProvider::default().build()])
        .commit()
        .unwrap();

    let session = Session::builder()
        .unwrap()
        .with_optimization_level(ort::GraphOptimizationLevel::Level1)
        .unwrap()
        .with_intra_threads(1)
        .unwrap()
        .with_inter_threads(1)
        .unwrap()
        .with_model_from_memory(onnx_file)
        .unwrap();

    let mut times = vec![1.0f32; n];

    for i in 0..n {
        // create input tensor (256x256x3, 1.0f32)
        let input = Array::<f32, _>::zeros((1, 256, 256, 3));

        let start = SystemTime::now();
        let outputs: ort::SessionOutputs = session
            .run(ort::inputs!["input_12" => input.view()].unwrap())
            .unwrap();
        let end = SystemTime::now();

        // add duration to times
        let duration = end.duration_since(start).unwrap().as_millis() as f32;
        times[i] = duration;
    }

    println!(
        "ORT: Average inference time: {} ms",
        times.iter().sum::<f32>() / n as f32
    );

    // WONNX
    //pollster::block_on(run_wonnx(onnx_file, n));
}
