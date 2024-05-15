use nokhwa::pixel_format::RgbFormat;
use nokhwa::{
    native_api_backend,
    pixel_format::RgbAFormat,
    query,
    utils::{
        frame_formats, yuyv422_predicted_size, CameraFormat, CameraIndex, FrameFormat,
        RequestedFormat, RequestedFormatType, Resolution,
    },
    Buffer, CallbackCamera, Camera,
};
use psychophysics::{
    include_image, loop_frames, start_experiment,
    visual::geometry::{Rectangle, Size, Transformation2D},
    visual::stimuli::{ImageStimulus, Pattern},
    visual::{stimuli::TextStimulus, Window},
};

fn show_image(window: Window) {
    // create a 640, height: 480 image
    let thatcher = image::DynamicImage::new_rgb8(1920, 1080);

    // create image stimulus
    let mut image_stim = ImageStimulus::new(&window, thatcher, Rectangle::FULLSCREEN);
    let image_stim_clone = image_stim.clone();
    // list all available cameras
    let cameras = nokhwa::query(nokhwa::native_api_backend().unwrap()).unwrap();
    // [print the cameras]
    println!("Number of cameras: {}", cameras.len());
    for camera in cameras {
        println!("camera: {}", camera);
    }
    let index = CameraIndex::Index(0);
    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let camera_format =
        CameraFormat::new(Resolution::new(1920, 1080), FrameFormat::RAWRGB, 30);

    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format));

    // make the camera
    let mut camera = CallbackCamera::new(index, requested, move |frame: Buffer| {
        // decode the buffer to RGB
        // print bit depth
        let bit_depth = ((frame.buffer().len() as f32) * 8.0)
            / (frame.resolution().width() * frame.resolution().height()) as f32;
        println!(
            "there are {} bits per pixel (width: {}, height: {})",
            bit_depth,
            frame.resolution().width(),
            frame.resolution().height()
        );

        let raw_buffer = frame.buffer().to_vec();

        // let frame =
        //     Buffer::new(frame.resolution(), raw_buffer.as_slice(), FrameFormat::YUYV);

        // let decoded: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = frame
        //     .decode_image::<RgbFormat>()
        //     .expect("failed to decode frame");

        // create image from buffer (assume the buffer is in RGB format)
        let decoded = image::ImageBuffer::from_raw(
            frame.resolution().width(),
            frame.resolution().height(),
            raw_buffer,
        )
        .expect("failed to create image from buffer");
        // update the image
        image_stim_clone.set_image(image::DynamicImage::ImageRgb8(decoded));
    })
    .unwrap();

    // start the camera
    camera.open_stream().unwrap();

    // create text stimulus
    let text_stim = TextStimulus::new(
        &window,
        "Ding Dong!",
        Rectangle::new(
            Size::Pixels(-250.0),
            Size::Pixels(-250.0),
            Size::Pixels(500.0),
            Size::Pixels(500.0),
        ),
    );

    // set color
    text_stim.set_color(psychophysics::visual::color::RED);
    let mut angle = 0.0;

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {

        // transform
        image_stim.set_transformation(Transformation2D::RotationCenter(angle));
        //video_stim.set_transformation(Transformation2D::RotationCenter(-angle));

        // add stimuli to frame
        frame.add(&image_stim);
        frame.add(&text_stim);

        angle += 0.5;
    });

    // close window
    window.close();
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
