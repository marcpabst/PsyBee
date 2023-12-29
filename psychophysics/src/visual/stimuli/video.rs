use super::image::{ImageStimulusImplementation, ImageStimulusParams};
use super::{
    super::geometry::{Rectangle, Size, ToVertices},
    super::pwindow::Window,
    base::{BaseStimulus, BaseStimulusImplementation, BaseStimulusParams},
};
use bytemuck::{Pod, Zeroable};
use futures_lite::future::block_on;
use half::f16;
use image;
use js_sys::Promise;
use rodio::queue;
use std::borrow::Cow;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wgpu::{Device, Queue, ShaderModule};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct VideoStimulusParams {
    s: f32,
}

pub struct VideoStimulusShader {
    pub(crate) window: Window,
    pub(crate) shader: ShaderModule,
    html_video_id: String,
    html_canvas_id: String,
    image: image::DynamicImage,
}

// TODO: make this a macro
impl BaseStimulusParams for VideoStimulusParams {}

pub type VideoStimulus<'a> =
    BaseStimulus<Rectangle, VideoStimulusShader, VideoStimulusParams>;

impl VideoStimulus<'_> {
    /// Create a new image stimulus.
    pub fn new(window: &Window, video_url: String) -> Self {
        let window = window.clone();
        window.clone().run_on_render_thread(|| async move {
            // create a shape that fills the screen
            let shape = Rectangle::new(
                -Size::ScreenWidth(0.5),
                -Size::ScreenHeight(0.5),
                Size::ScreenWidth(1.0),
                Size::ScreenHeight(1.0),
            );

            let window_state = window.get_window_state_blocking();
            let device = &window_state.device;

            let shader =
                VideoStimulusShader::new(&window, &device, video_url).await;

            let params = VideoStimulusParams { s: 3.0 };

            drop(window_state); // this prevent a deadlock (argh, i'll have to refactor this)

            // create texture size based on image size
            let texture_size = wgpu::Extent3d {
                width: shader.image.width(),
                height: shader.image.height(),
                depth_or_array_layers: 1,
            };

            let mut out = BaseStimulus::create(
                &window,
                shader,
                shape,
                params,
                Some(texture_size),
            );

            out
        })
    }
}

impl BaseStimulusImplementation<VideoStimulusParams>
    for ImageStimulusImplementation
{
    fn update(
        &mut self,
        params: &mut VideoStimulusParams,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Option<Vec<u8>> {
        // nothing to do here
        None
    }
    fn get_shader(&self) -> &ShaderModule {
        &self.shader
    }
}

/// This is a hack to get the video to play in the background. In theory, we should be able to
/// just create a HTMLVideoElement and access it from the render thread. However, this does not
/// work as web_sys will somehow lose the reference to the video element. To work around this
/// problem, we store the video element in the DOM and access it using its id.
impl VideoStimulusShader {
    pub async fn new(
        window: &Window,
        device: &Device,
        video_url: String,
    ) -> Self {
        let shader: ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shaders/image.wgsl"
                ))),
            });

        let web_window = web_sys::window().expect("no global `window` exists");
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
        let onload_future = JsFuture::from(Promise::new(&mut |resolve, _| {
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

        // convert to image
        let image = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(1280, 720, image_data).unwrap(),
        );

        // play the video in the canvas
        video.play().unwrap();

        Self {
            window: window.clone(),
            shader,
            html_video_id: video.id(),
            html_canvas_id: canvas.id(),
            image,
        }
    }
}

impl Drop for VideoStimulusShader {
    fn drop(&mut self) {
        let video_id = self.html_video_id.clone();
        self.window.run_on_render_thread(|| async move {
            log::info!("Removing video element with id {}", &video_id);
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

impl BaseStimulusImplementation<VideoStimulusParams> for VideoStimulusShader {
    fn update(
        &mut self,
        params: &mut VideoStimulusParams,
        width_mm: f64,
        viewing_distance_mm: f64,
        width_px: i32,
        height_px: i32,
    ) -> Option<Vec<u8>> {
        //  update the texture
        let video_id = self.html_video_id.clone();
        let canvas_id = self.html_canvas_id.clone();
        let window = self.window.clone();

        log::info!("Updating video element with id {}", &video_id);
        let web_window = web_sys::window().expect("no global `window` exists");
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
        Some(image_data)
    }
    fn get_shader(&self) -> &ShaderModule {
        &self.shader
    }
}
