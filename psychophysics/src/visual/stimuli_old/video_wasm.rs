use super::{
    super::geometry::{Rectangle, Size, ToVertices},
    super::pwindow::Window,
    base::{BaseStimulus, BaseStimulusImplementation},
};

use image;
use js_sys::Promise;

use std::borrow::Cow;
use wasm_bindgen::JsCast;

use wasm_bindgen_futures::JsFuture;
use wgpu::{Device, ShaderModule};

pub struct VideoStimulusImplementation {
    pub(crate) window: Window,
    html_video_id: String,
    html_canvas_id: String,
    shape: Rectangle,
    frame: image::DynamicImage,
}

pub type VideoStimulus<'a> =
    BaseStimulus<VideoStimulusImplementation>;

impl VideoStimulus<'_> {
    pub fn new(window: &Window, video_url: String) -> Self {
        // create a shape that fills the screen
        let shape = Rectangle::new(
            -Size::ScreenWidth(0.5),
            -Size::ScreenHeight(0.5),
            Size::ScreenWidth(1.0),
            Size::ScreenHeight(1.0),
        );

        Self::_new(window, video_url, shape)
    }
    pub fn new_with_rectangle(
        window: &Window,
        video_url: String,
        shape: Rectangle,
    ) -> Self {
        Self::_new(window, video_url, shape)
    }
    /// Create a new image stimulus.
    fn _new(
        window: &Window,
        video_url: String,
        shape: Rectangle,
    ) -> Self {
        let window = window.clone();
        window.clone().run_on_render_thread(|| async move {
            let window_state = window.get_window_state_blocking();
            let device = &window_state.device;

            let implementation = VideoStimulusImplementation::new(
                &window, &device, video_url, shape,
            )
            .await;

            // create texture size based on image size
            let _texture_size = wgpu::Extent3d {
                width: implementation.frame.width(),
                height: implementation.frame.height(),
                depth_or_array_layers: 1,
            };

            let out = BaseStimulus::create(
                &window,
                &window_state,
                implementation,
            );

            out
        })
    }
}

/// This is a hack to get the video to play in the background. In theory, we should be able to
/// just create a HTMLVideoElement and access it from the render thread. However, this does not
/// work as web_sys will somehow lose the reference to the video element. To work around this
/// problem, we store the video element in the DOM and access it using its id.
impl VideoStimulusImplementation {
    pub async fn new(
        window: &Window,
        device: &Device,
        video_url: String,
        shape: Rectangle,
    ) -> Self {
        let _shader: ShaderModule = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                    include_str!("shaders/image.wgsl"),
                )),
            },
        );

        let web_window =
            web_sys::window().expect("no global `window` exists");
        let web_document = web_window
            .document()
            .expect("should have a document on window");
        let video = web_document
            .create_element("video")
            .unwrap()
            .dyn_into::<web_sys::HtmlVideoElement>()
            .unwrap();

        let canvas = web_document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        // Create a random id for the video element using fastrand
        let rand_id = (0..10)
            .map(|_| fastrand::alphanumeric())
            .collect::<String>();

        let video_id = format!("video_{}", rand_id);
        let canvas_id = format!("canvas_{}", rand_id);

        video.set_id(&video_id.as_str());

        canvas.set_id(&canvas_id.as_str());

        // set css class to hidden
        video.set_class_name("hidden");
        canvas.set_class_name("hidden");

        // add the video to the body
        web_document.body().unwrap().append_child(&video).unwrap();
        web_document.body().unwrap().append_child(&canvas).unwrap();

        log::info!("Loading video from {} ...", video_url);

        // set the source
        video.set_src(video_url.as_str());

        // convert the onload event to a future
        let onload_future =
            JsFuture::from(Promise::new(&mut |resolve, _| {
                video.set_oncanplay(Some(&resolve));
            }));

        log::info!("Waiting for video to load ...");

        // load the video
        video.load();

        // wait for the onload event to complete
        onload_future.await.unwrap();

        // register an o

        // draw the video to the canvas
        video.set_width(1280); // TODO: make this configurable
        video.set_height(720); // TODO: make this configurable
        canvas.set_width(1280); // TODO: make this configurable
        canvas.set_height(720); // TODO: make this configurable
        let ctx = canvas
            .get_context_with_context_options(
                "2d",
                &js_sys::eval("({ willReadFrequently: true })")
                    .unwrap(),
            )
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        // draw the video to the canvas
        ctx.draw_image_with_html_video_element_and_dw_and_dh(
            &video, 0.0, 0.0, 1280.0, 720.0,
        );

        // get the image data
        let image_data: Vec<u8> = ctx
            .get_image_data(0.0, 0.0, 1280.0, 720.0)
            .unwrap()
            .data()
            .to_vec();

        // convert to image
        let image = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(1280, 720, image_data)
                .unwrap(),
        );

        // play the video in the canvas
        video.play().unwrap();

        Self {
            window: window.clone(),
            html_video_id: video.id(),
            html_canvas_id: canvas.id(),
            shape,
            frame: image,
        }
    }
}

impl Drop for VideoStimulusImplementation {
    fn drop(&mut self) {
        let video_id = self.html_video_id.clone();
        self.window.run_on_render_thread(|| async move {
            log::info!(
                "Removing video element with id {}",
                &video_id
            );
            let web_window =
                web_sys::window().expect("no global `window` exists");
            let web_document = web_window
                .document()
                .expect("should have a document on window");
            let video = web_document
                .get_element_by_id(&video_id)
                .unwrap()
                .dyn_into::<web_sys::HtmlVideoElement>()
                .unwrap();
            video.remove();
        });
    }
}

impl BaseStimulusImplementation for VideoStimulusImplementation {
    fn update(
        &mut self,
        _screen_width_mm: f64,
        _viewing_distance_mm: f64,
        _screen_width_px: u32,
        _screen_height_px: u32,
    ) -> (Option<&[u8]>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>)
    {
        //  update the texture
        let video_id = self.html_video_id.clone();
        let canvas_id = self.html_canvas_id.clone();
        let _window = self.window.clone();

        log::info!("Updating video element with id {}", &video_id);
        let web_window =
            web_sys::window().expect("no global `window` exists");
        let web_document = web_window
            .document()
            .expect("should have a document on window");
        let video = web_document
            .get_element_by_id(&video_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlVideoElement>()
            .unwrap();

        let canvas = web_document
            .get_element_by_id(&canvas_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        // draw the video to the canvas
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // draw the video to the canvas
        ctx.draw_image_with_html_video_element_and_dw_and_dh(
            &video, 0.0, 0.0, 1280.0, 720.0,
        );

        // get the image data
        let image_data: Vec<u8> = ctx
            .get_image_data(0.0, 0.0, 1280.0, 720.0)
            .unwrap()
            .data()
            .to_vec();

        // // convert to image
        // let image = image::DynamicImage::ImageRgba8(
        //     image::RgbaImage::from_raw(200, 200, image_data).unwrap(),
        // );

        // return
        (None, None, Some(image_data))
    }

    fn get_fragment_shader_code(&self) -> String {
        "
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(texture, texture_sampler, in.tex_coords);
        }
        "
        .to_string()
    }
    fn get_texture_data(&self) -> Option<Vec<u8>> {
        // convert from rgba to bgra
        let texture_data: Vec<u8> = self
            .frame
            .to_rgba8()
            .chunks_exact(4)
            .flat_map(|chunk| {
                [
                    chunk[2], // r
                    chunk[1], // g
                    chunk[0], // b
                    chunk[3], // a
                ]
            })
            .collect();

        Some(texture_data)
    }

    fn get_texture_size(&self) -> Option<wgpu::Extent3d> {
        Some(wgpu::Extent3d {
            width: self.frame.width(),
            height: self.frame.height(),
            depth_or_array_layers: 1,
        })
    }

    fn get_geometry(&self) -> Box<dyn ToVertices> {
        Box::new(self.shape.clone())
    }
}
