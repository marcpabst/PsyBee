// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
// first camera in system

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
fn main() {
    let index = CameraIndex::Index(0);
    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let camera_format =
        CameraFormat::new(Resolution::new(1920, 1080), FrameFormat::YUYV, 30);

    let mut requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format));

    // make the camera
    let mut camera = CallbackCamera::new(index, requested, move |frame: Buffer| {
        // decode the buffer to RGB
        let frame = Buffer::new(frame.resolution(), frame.buffer(), FrameFormat::YUYV);

        let decoded: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = frame
            .decode_image::<RgbFormat>()
            .expect("failed to decode frame");
        // print the resolution
        println!("width: {}, height: {}", decoded.width(), decoded.height());
    })
    .unwrap();

    camera.open_stream().unwrap();

    // stall the thread for 10s
    std::thread::sleep(std::time::Duration::from_secs(10));
}
