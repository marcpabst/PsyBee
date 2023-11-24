use tract_onnx::prelude::*;
use web_time::SystemTime;

fn main() {
    // load onnx model at compile time
    let onnx_file = include_bytes!("face_landmarks_detector.onnx");
    let mut onnx_reader = std::io::Cursor::new(onnx_file);

    let model = tract_onnx::onnx()
        .model_for_read(&mut onnx_reader)
        .unwrap()
        // optimize the model
        .into_optimized()
        .unwrap()
        // make the model runnable and fix its inputs and outputs
        .into_runnable()
        .unwrap();

    let N = 10;

    let mut times = vec![1.0f32; N];

    for i in 0..N {
        // create input tensor (256x256x3, 1.0f32)
        let dummy_input = Tensor::zero_dt(DatumType::F32, &[1, 256, 256, 3]).unwrap();

        let start = SystemTime::now();
        let outputs = model.run(tvec![dummy_input.into()]).unwrap();
        let end = SystemTime::now();

        // add duration to times
        let duration = end.duration_since(start).unwrap().as_millis() as f32;
        times[i] = duration;
    }

    println!(
        "Average inference time: {} ms",
        times.iter().sum::<f32>() / N as f32
    );
}
