use std::sync::Arc;

use crate::{input::Key, log_extra, request_animation_frame, PFutureReturns};
use async_std::sync::{Mutex, MutexGuard};
use async_std::task::{self};
use flume::{Receiver, Sender};
use futures_lite::future::block_on;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::console;

use super::{Color, Renderable};

pub struct PWindow {
    // the winit window
    pub window: winit::window::Window,
    // the wgpu device
    pub device: wgpu::Device,
    // the wgpu instance
    pub instance: wgpu::Instance,
    // the wgpu adapter
    pub adapter: wgpu::Adapter,
    // the wgpu queue
    pub queue: wgpu::Queue,
    // the wgpu surface
    pub surface: wgpu::Surface,
    // the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Clone)]
pub struct WindowHandle {
    pub pw: Arc<Mutex<PWindow>>,
    // broadcast receiver for keyboard events
    pub keyboard_receiver: async_broadcast::InactiveReceiver<winit::event::KeyboardInput>,
    // channel for frame submission
    pub frame_sender: Sender<Arc<Mutex<Frame>>>,
    pub frame_receiver: Receiver<Arc<Mutex<Frame>>>,
    // channel for frame completion
    pub frame_ok_sender: Sender<bool>,
    pub frame_ok_receiver: Receiver<bool>,
}

impl WindowHandle {
    pub async fn get_window(&self) -> MutexGuard<PWindow> {
        return self.pw.lock().await;
    }

    pub async fn wait_for_keypress<T, I>(
        // the PWindow behind a mutex
        &self,
        keys: T,
    ) -> Result<PFutureReturns, anyhow::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Key>,
    {
        let start: web_time::Instant = web_time::Instant::now();

        let mut keyboard_receiver = self.keyboard_receiver.activate_cloned();

        let key_vec: Vec<Key> = keys.into_iter().map(|k| k.into()).collect();

        let kc: winit::event::VirtualKeyCode;
        loop {
            // wait for buttons pres
            let e = keyboard_receiver
                .recv()
                .await
                .map_err(|_| anyhow::anyhow!("Failed to receive keypress from channel"))?;

            // check if keypress matches any of the keys
            if key_vec.contains(&e.virtual_keycode.unwrap().into()) || key_vec.is_empty() {
                kc = e.virtual_keycode.unwrap();
                break;
            }
        }

        return Ok(PFutureReturns::KeyPress((
            kc.into(),
            web_time::Instant::now().duration_since(start),
        )));
    }

    pub async fn wait_for_any_keypress(&self) -> Result<PFutureReturns, anyhow::Error> {
        let empty_vec: Vec<Key> = Vec::new();
        return self.wait_for_keypress(empty_vec).await;
    }

    pub async fn submit_frame(&self, frame: Frame) {
        let frame_sender = self.frame_sender.clone();
        let frame_ok_receiver = self.frame_ok_receiver.clone();

        // submit frame to channel
        frame_sender.send_async(Arc::new(Mutex::new(frame))).await;

        // wait for frame to be consumed
        frame_ok_receiver.recv_async().await;
    }

    pub async fn render_task(self) {
        let window_handle = self;

        // get rx and tx from handle
        let tx = window_handle.frame_ok_sender.clone();
        let rx = window_handle.frame_receiver.clone();

        // on wasm, we register our own requestAnimationFrame callback in a separate task
        #[cfg(target_arch = "wasm32")]
        {
            log_extra!(
                "Task RENDER running on thread {:?}",
                std::thread::current().id()
            );

            // here, we create a closure that will be called by requestAnimationFrame
            let f = Rc::new(RefCell::new(None));
            let g = f.clone();

            *g.borrow_mut() = Some(Closure::new(move || {
                // Set the body's text content to how many times this
                // requestAnimationFrame callback has fired.

                // Schedule ourself for another requestAnimationFrame callback.
                request_animation_frame(f.borrow().as_ref().unwrap());

                // check if there is a frame available
                let try_frame = rx.try_recv();

                if try_frame.is_ok() {
                    let frame = try_frame.unwrap();
                    // acquire lock on frame
                    let mut frame = block_on(frame.lock());

                    // acquire lock on window
                    let window_lock = block_on(window_handle.get_window());

                    let suface_texture: wgpu::SurfaceTexture = window_lock
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = suface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = window_lock
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // clear the frame
                    {
                        // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                        // using the LoadOp::Clear option)
                        let rpass = &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(frame.bg_color.into()),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                    }

                    frame.prepare(
                        &window_lock.device,
                        &window_lock.queue,
                        &view,
                        &window_lock.config,
                    );

                    frame.render(&mut encoder, &view);

                    window_lock.queue.submit(Some(encoder.finish()));
                    suface_texture.present();

                    // notify sender that frame has been consumed
                    let _ = tx.try_send(true);
                }
            }));

            request_animation_frame(g.borrow().as_ref().unwrap());
        }
        // on native, we submit frames when they are ready
        #[cfg(not(target_arch = "wasm32"))]
        {
            log_extra!(
                "Task RENDER running on thread {:?}",
                std::thread::current().id()
            );

            loop {
                // wait for frame to be submitted
                let frame = rx.recv_async().await.unwrap();

                // acquire lock on frame
                let mut frame = block_on(frame.lock());

                // acquire lock on window
                let window_lock = window_handle.get_window().await;

                let suface_texture = window_lock
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = suface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = window_lock
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                // clear the frame
                {
                    // clear the frame (once the lifetime annoyance is fixed, this can be removed only a single render pass is needed
                    // using the LoadOp::Clear option)
                    let _rpass = &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(frame.bg_color.into()),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }
                frame.prepare(
                    &window_lock.device,
                    &window_lock.queue,
                    &view,
                    &window_lock.config,
                );
                frame.render(&mut encoder, &view);

                window_lock.queue.submit(Some(encoder.finish()));
                suface_texture.present();

                // notify sender that frame has been consumed
                let _ = tx.send_async(true).await;
            }
        }
    }
}

pub struct Frame {
    renderables: Arc<Mutex<Vec<Box<dyn Renderable>>>>,
    pub bg_color: wgpu::Color,
}

impl Renderable for Frame {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
    ) -> () {
        // call prepare() on all renderables
        for renderable in &mut (block_on(self.renderables.lock())).iter_mut() {
            renderable.prepare(device, queue, view, config);
        }
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        // call render() on all renderables
        for renderable in &mut (block_on(self.renderables.lock())).iter_mut() {
            renderable.render(enc, view);
        }
    }
}

impl Frame {
    // create a new frame
    pub fn new() -> Self {
        Self {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color: Color::RGB {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }
            .into(),
        }
    }

    pub fn new_with_bg_color(bg_color: wgpu::Color) -> Self {
        Self {
            renderables: Arc::new(Mutex::new(Vec::new())),
            bg_color,
        }
    }

    // add a renderable to the frame
    pub fn add(&mut self, renderable: &(impl Renderable + Clone + 'static)) -> () {
        let renderable = Box::new(renderable.clone());
        block_on(self.renderables.lock()).push(renderable);
    }
}

// mark Frame as Send and Sync
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}
