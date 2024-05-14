// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use byte_slice_cast::AsSliceOf;
use gstreamer;
use gstreamer::glib::PropertyGet;
use gstreamer::DeviceMonitor;
use gstreamer::{element_error, prelude::*};
use image::ImageBuffer;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Debug, EnumString)]
pub enum Framerate {
    Exact(f64),
    Range(f64, f64),
}

#[derive(Copy, Clone, Debug)]
pub enum CameraAPI {
    // Linux
    V4L2,
    // MacOS
    AVFoundation,
    // Windows
    WindowsMediaFoundation,
}

#[derive(Copy, Clone, Debug, EnumString, Display)]
pub enum VideoFormat {
    // YUV formats
    YUYV,
    UYVY,
    YVYU,
    VYUY,
    YUY2,
    // RGB formats
    RGB,
    // Bayer formats
    RGGB,
    GRBG,
    GBRG,
    BGGR,
    BGRA,
    // Other formats
    GREY,
    NV12,
    NV21,
}

pub trait VideoModeTrait: Clone {}

#[derive(Copy, Clone, Debug)]
pub struct CameraMode {
    width: u32,
    height: u32,
    framerate: Framerate,
    frame_format: VideoFormat,
}
impl VideoModeTrait for CameraMode {}

#[derive(Copy, Clone, Debug)]
pub struct VideoMode {
    width: u32,
    height: u32,
    framerate: Framerate,
    frame_format: VideoFormat,
}
impl VideoModeTrait for VideoMode {}

pub trait VideoSourceTrait: Clone {}

#[derive(Clone, Debug)]
pub struct Camera {
    device_name: String,
    human_name: String,
    device_id: u32,
    modes: Vec<CameraMode>,
    api: CameraAPI,
}

impl VideoSourceTrait for Camera {}

#[derive(Clone, Debug)]
pub struct CameraStream {
    camera: Camera,
    mode: CameraMode,
    output_format: VideoFormat,
}

impl Camera {
    pub fn modes(&self) -> Vec<CameraMode> {
        self.modes.clone()
    }

    pub fn open_with_callback<F>(
        &self,
        mode: &CameraMode,
        callback: F,
    ) -> Result<(), String>
    where
        F: Fn(VideoFrame) + Send + 'static,
    {
        let mode = mode.clone();
        // allocate CPU memory for the frames
        let local_buffer = Arc::new(Mutex::new(vec![0u8]));
        let local_buffer_clone = local_buffer.clone();

        let sink = gstreamer::ElementFactory::make("appsink").build().expect(
            "Could not create appsink. Make sure you have the `bad` plugin as it is not a core plugin",
        );

        // Cseate a new source element
        let sourcename = match self.api {
            CameraAPI::V4L2 => "v4l2src",
            CameraAPI::AVFoundation => "avfvideosrc",
            CameraAPI::WindowsMediaFoundation => "ksvideosrc",
        };

        let src = gstreamer::ElementFactory::make(sourcename)
            .property("device-index", &(self.device_id as i32))
            .build()
            .expect("Could not create video source");

        let capsfilter = gstreamer::ElementFactory::make("capsfilter")
            .build()
            .expect("Could not create capsfilter");
        let caps = gstreamer::Caps::builder("video/x-raw")
            .field("format", &mode.frame_format.to_string())
            .field("width", mode.width as i32)
            .field("height", mode.height as i32)
            .build();

        // print caps
        println!("caps: {:?}", caps);

        capsfilter.set_property("caps", &caps);

        // create pipeline
        let pipeline = gstreamer::Pipeline::new();

        pipeline
            .add_many(&[&src, &capsfilter, &sink])
            .expect("Unable to add elements to pipeline");

        src.link(&capsfilter)
            .expect("Unable to link elements in pipeline");
        capsfilter
            .link(&sink)
            .expect("Unable to link elements in pipeline");

        let appsink = sink
            .dynamic_cast::<gstreamer_app::AppSink>()
            .expect("Sink element is expected to be an appsink!");

        // Getting data out of the appsink is done by setting callbacks on it.
        // The appsink will then call those handlers, as soon as data is available.
        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                // Add a handler to the "new-sample" signal.
                .new_sample(move |appsink| {
                    // print size of the buffer
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or_else(|| {
                        element_error!(
                            appsink,
                            gstreamer::ResourceError::Failed,
                            ("Failed to get buffer from appsink")
                        );
                        gstreamer::FlowError::Error
                    })?;

                    let map = buffer.map_readable().map_err(|_| {
                        element_error!(
                            appsink,
                            gstreamer::ResourceError::Failed,
                            ("Failed to map buffer readable")
                        );
                        gstreamer::FlowError::Error
                    })?;

                    let data = map.as_slice_of::<u8>().unwrap();

                    // call the callback
                    callback(VideoFrame {
                        data: data.to_vec(),
                        width: mode.width,
                        height: mode.height,
                        format: mode.frame_format,
                    });

                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");

        let bus = pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        loop {
            use gstreamer::MessageView;

            match bus.pop() {
                Some(msg) => match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {} ({:?})",
                            err.src().map(|s| s.path_string()),
                            err.error(),
                            err.debug()
                        );
                        break;
                    }
                    _ => (),
                },
                None => (),
            }
        }

        Ok(())
    }
}

// impl pretty printing for Camera
impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{{device_name: {}, human_name: {}, device_id: {}, modes: {:?}}}",
            self.device_name, self.human_name, self.device_id, self.modes
        )
    }
}

#[derive(Clone, Debug)]
pub struct VideoFile {
    filename: String,
}
#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
}

pub struct CameraManager {
    cameras: Vec<Camera>,
}

impl CameraManager {
    pub fn new() -> Self {
        Self {
            cameras: Self::fetch_cameras(),
        }
    }

    fn fetch_cameras() -> Vec<Camera> {
        // initialize gstreamer
        gstreamer::init().unwrap();

        // create a device monitor
        let monitor: DeviceMonitor = DeviceMonitor::new();

        // add a filter to only show video devices
        monitor.add_filter(Some("Video/Source"), None);

        // start the monitor
        monitor.start();

        // get the list of devices
        let devices = monitor.devices();

        // create a vector to store the cameras
        let mut cameras: Vec<Camera> = Vec::new();

        // iterate over the devices
        for (i, device) in devices.iter().enumerate() {
            // get the device name
            let device_name = device.name().to_string();
            let human_name = device.display_name().to_string();

            // get the device id
            let device_id = i as u32;

            // get the device caps
            let device_caps = device.caps().unwrap();
            // get number of caps
            let num_caps = device_caps.size();

            // get properties of camera
            let device_properties = device.properties().unwrap();
            // print properties
            println!("device properties: {:?}", device_properties);

            // stall thread for 1s
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // get the api
            let api_string = device_properties
                .get::<String>("device.api")
                .unwrap_or_else(|e| {
                    panic!("Error getting gstreamer api from device properties: {}", e)
                });

            let api = match api_string.as_str() {
                "v4l2" => CameraAPI::V4L2,
                "avf" => CameraAPI::AVFoundation,
                "windows-media-foundation" => CameraAPI::WindowsMediaFoundation,
                _ => panic!("Unknown gstreamer api: {}", api_string),
            };

            // create a vector to store the modes
            let mut modes: Vec<CameraMode> = Vec::new();
            for i in 0..num_caps {
                // get the caps structure
                let caps_struct = device_caps.structure(i).unwrap();

                // get the caps name
                let caps_name = caps_struct.name();

                // get the caps width
                let caps_width = caps_struct.get::<i32>("width").unwrap();

                // get the caps height
                let caps_height = caps_struct.get::<i32>("height").unwrap();

                // get the caps framerate
                let caps_framerate = caps_struct
                    .get::<gstreamer::FractionRange>("framerate")
                    .unwrap();

                let framerate_min = caps_framerate.min();
                let framerate_min =
                    framerate_min.denom() as f64 / framerate_min.numer() as f64;
                let framerate_max = caps_framerate.max();
                let framerate_max =
                    framerate_max.denom() as f64 / framerate_max.numer() as f64;

                let framerate = if framerate_min == framerate_max {
                    Framerate::Exact(framerate_min)
                } else {
                    Framerate::Range(framerate_min, framerate_max)
                };

                // get list of caps formats
                let caps_formats = caps_struct
                    .get::<gstreamer::List>("format")
                    .unwrap_or_else(|e| {
                        let v = vec![caps_struct.get::<String>("format").unwrap()];
                        gstreamer::List::new(v)
                    });

                // convert to vector of strings
                let caps_formats_vec = caps_formats
                    .iter()
                    .map(|x| {
                        VideoFormat::from_str(x.get::<String>().unwrap().as_str())
                            .unwrap_or_else(|e| {
                                panic!(
                                    "Error converting caps format {:?} to VideoFormat: {}",
                                    e,
                                    x.get::<String>().unwrap()
                                )
                            })
                    })
                    .collect::<Vec<VideoFormat>>();

                for format in caps_formats_vec {
                    // create a camera mode
                    let mode = CameraMode {
                        width: caps_width as u32,
                        height: caps_height as u32,
                        framerate: framerate,
                        frame_format: format,
                    };

                    // add the mode to the vector
                    modes.push(mode);
                }
            }

            // create a camera
            let camera = Camera {
                device_name,
                human_name,
                device_id,
                modes: modes,
                api: api,
            };

            // add the camera to the vector
            cameras.push(camera);
        }

        // return the vector
        cameras
    }

    pub fn cameras(&self) -> Vec<Camera> {
        self.cameras.clone()
    }

    pub fn update(&mut self) {
        self.cameras = Self::fetch_cameras();
    }

    pub fn get_camera(&self, device_id: u32) -> Option<Camera> {
        self.cameras
            .iter()
            .find(|x| x.device_id == device_id)
            .cloned()
    }
}

// comvert from video format to image format
impl From<VideoFrame> for ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    fn from(video_frame: VideoFrame) -> Self {
        let width = video_frame.width;
        let height = video_frame.height;
        let format = video_frame.format;
        let data = video_frame.data;

        let mut image_buffer = ImageBuffer::new(width, height);

        match format {
            VideoFormat::RGB => {
                for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
                    let index = (y * width + x) as usize * 3;
                    let r = data[index];
                    let g = data[index + 1];
                    let b = data[index + 2];
                    *pixel = image::Rgb([r, g, b]);
                }
            }
            VideoFormat::BGRA => {
                for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
                    let index = (y * width + x) as usize * 4;
                    let b = data[index];
                    let g = data[index + 1];
                    let r = data[index + 2];
                    *pixel = image::Rgb([r, g, b]);
                }
            }
            VideoFormat::GREY => {
                for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
                    let index = (y * width + x) as usize;
                    let g = data[index];
                    *pixel = image::Rgb([g, g, g]);
                }
            }
            _ => panic!("Unsupported video format: {:?}", format),
        }

        image_buffer
    }
}
