use crate::utils::BlockingLock;
use crate::visual::geometry::Size;
use crate::visual::Renderable;
use async_lock::Mutex;
use async_trait::async_trait;





use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wgpu::TextureFormat;

use super::super::geometry::ToVertices;
use super::super::geometry::Transformation2D;
use super::super::geometry::Vertex;
use super::super::pwindow::Window;
use super::super::pwindow::WindowState;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, SurfaceConfiguration};

pub trait BaseStimulusImplementation {
    /// Called when `prepare()` is called on the stimulus.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters that are passed to the fragment shader.
    /// * `screen_width_mm` - The width of the screen in mm.
    /// * `viewing_distance_mm` - The viewing distance in mm.
    /// * `screen_width_px` - The width of the screen in pixels.
    /// * `screen_height_px` - The height of the screen in pixels.
    ///
    /// # Returns
    ///
    /// * `Option<P>` - If the stimulus has changed, this should return `Some(params as bytes)`, otherwise `None`.
    /// * `Option<Box<dyn ToVertices>>` - If the stimulus has changed, this should return `Some(geometry)`, otherwise `None`.
    /// * `Option<Vec<u8>>` - If the stimulus has changed, this should return `Some(texture_data)`, otherwise `None`.
    fn update(
        &mut self,
        _screen_width_mm: f64,
        _viewing_distance_mm: f64,
        _screen_width_px: u32,
        _screen_height_px: u32,
    ) -> (Option<&[u8]>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>) {
        (None, None, None)
    }
    /// Returns the the function that will be used as the fragment shader.
    fn get_fragment_shader_code(&self) -> String;

    /// Returns the data that will be used to initialize the uniform buffer.
    ///
    /// # Caveat
    /// Make sure that the memory layout of whatever you convert
    /// to bytes is the same as the memory layout of the uniform buffer
    /// in the shader. For structs, this means that you should use the
    /// `#[repr(C)]` attribute. See https://docs.rs/bytemuck/1.5.0/bytemuck/
    /// for more information.
    fn get_uniform_buffer_data(&self) -> Option<&[u8]> {
        None
    }

    /// Returns the texture size (if any) that will be used for the stimulus.
    /// If no texture is specified, this should return `None` (default).
    fn get_texture_size(&self) -> Option<wgpu::Extent3d> {
        None
    }

    /// Returns the texture data (if any) that will be used to initialize the texture.
    /// If no texture is specified, this should return `None` (default).
    fn get_texture_data(&self) -> Option<Vec<u8>> {
        None
    }

    // TODO: should be parametrized by the number of vertices to avoid buffer overflows at runtime
    /// Returns the geometry that will be used for the stimulus. By default, this is a rectangle
    /// that spans the entire screen.
    fn get_geometry(&self) -> Box<dyn ToVertices> {
        Box::new(super::super::geometry::Rectangle::new(
            Size::ScreenWidth(-0.5),
            Size::ScreenHeight(-0.5),
            Size::ScreenWidth(1.0),
            Size::ScreenHeight(1.0),
        ))
    }
}

/// A very flexible stimulus that allows a wide range of visual stimuli to be created.
///
/// The `BaseStimulusImplementation` will provide the fragment shader code,
/// the uniform buffer data, the texture size and data, and the geometry while
/// the BaseStimulus will take care of all the boilerplate code responsible for
/// setting up the rendering pipeline and uploading the data to the GPU.
/// It will also apply any transformations and ensure that the geometry is
/// converted to pixel coordinates before being uploaded to the GPU.
pub struct BaseStimulus<S: BaseStimulusImplementation> {
    /// The implementation of the stimulus.
    pub(crate) stimulus_implementation: Arc<Mutex<S>>,
    /// The rendering pipeline for the stimulus.
    pipeline: Arc<Mutex<wgpu::RenderPipeline>>,
    /// The window used to create the stimulus.
    window: Window,
    /// A `Transformation2D` that will be applied in the vertex shader.
    transforms: Arc<Mutex<Transformation2D>>,
    /// Vertex buffer that will be uploaded to the shader.
    vertex_buffer: Arc<Mutex<wgpu::Buffer>>,
    /// Number of vertices.
    n_vertices: AtomicUsize,
    /// Bind group 0.
    bind_group: Arc<Mutex<wgpu::BindGroup>>,
    /// Uniform buffer for the pixel shader paramters.
    uniform_buffer: Arc<Mutex<wgpu::Buffer>>,
    /// Unifrom buffer for the transformation matrix.
    transform_buffer: Arc<Mutex<wgpu::Buffer>>,
    /// (Optional) texture size.
    texture_size: Option<wgpu::Extent3d>,
    /// (Optional) texture.
    pub(crate) texture: Option<Arc<Mutex<wgpu::Texture>>>,
    /// (Optional) texture bind group.
    texture_bind_group: Option<Arc<Mutex<wgpu::BindGroup>>>,
}

// todo: this should be derived
unsafe impl<S: BaseStimulusImplementation> Send for BaseStimulus<S> {}

// same here
unsafe impl<S: BaseStimulusImplementation> Sync for BaseStimulus<S> {}

impl<S: BaseStimulusImplementation> Clone for BaseStimulus<S> {
    fn clone(&self) -> Self {
        Self {
            uniform_buffer: self.uniform_buffer.clone(),
            bind_group: self.bind_group.clone(),
            pipeline: self.pipeline.clone(),
            window: self.window.clone(),
            stimulus_implementation: self.stimulus_implementation.clone(),
            transforms: self.transforms.clone(),
            vertex_buffer: self.vertex_buffer.clone(),
            transform_buffer: self.transform_buffer.clone(),
            n_vertices: AtomicUsize::new(self.n_vertices.load(Ordering::Relaxed)),
            texture_size: self.texture_size.clone(),
            texture: self.texture.clone(),
            texture_bind_group: self.texture_bind_group.clone(),
        }
    }
}

#[async_trait(?Send)]
impl<S: BaseStimulusImplementation> Renderable for BaseStimulus<S> {
    async fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &super::super::pwindow::Window,
    ) -> () {
        let screen_width_mm = window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm = window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = config.width;
        let screen_height_px = config.height;

        let mut simp = self.stimulus_implementation.lock_blocking();
        // call the shader update function
        let (params, geometry, tex_data) = {
            simp.update(
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            )
        };

        // if the shader returned new texture data, update the texture
        if let Some(tex_data) = tex_data {
            let binding = self.texture.clone().expect(
                "Texture data was returned by the implementation but no texture was specified when creating the stimulus.",
            );
            let texture = binding.lock_blocking();
            self.set_texture(tex_data.as_slice(), &queue, &texture);
        }

        // if the shader returned new parameters, update the uniform buffer
        if let Some(params) = params {
            queue.write_buffer(&(self.uniform_buffer.lock_blocking()), 0, params);
        }

        // if the shader returned new geometry, update the vertex buffer
        if let Some(geometry) = geometry {
            let vertices = geometry.to_vertices_px(
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            );
            self.n_vertices.store(vertices.len(), Ordering::Relaxed);

            // update the vertex buffer
            queue.write_buffer(
                &(self.vertex_buffer.lock_blocking()),
                0,
                bytemuck::cast_slice(&vertices),
            );
        }

        // update the transform buffer
        // first get the transformation matrix from the window handle
        let win_transform =
            Window::transformation_matrix_to_ndc(screen_width_px, screen_height_px)
                .map(|x| x as f32);
        let stim_transform = (self.transforms.lock_blocking()).to_transformation_matrix(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        // cast to f32
        let transform = win_transform * stim_transform;

        queue.write_buffer(
            &(self.transform_buffer.lock_blocking()),
            0,
            bytemuck::cast_slice(transform.as_slice()),
        );
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        let pipeline = self.pipeline.lock_blocking();
        let n_vertices = self.n_vertices.load(Ordering::Relaxed);
        let texture_bind_group =
            if let Some(texture_bind_group) = &self.texture_bind_group {
                Some(texture_bind_group.lock_blocking())
            } else {
                None
            };

        let bind_group = self.bind_group.lock_blocking();
        let vertex_buffer = self.vertex_buffer.lock_blocking();
        {
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&pipeline);
            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            rpass.set_bind_group(0, &bind_group, &[]);

            // if a texture is specified, set the texture bind group
            if let Some(texture_bind_group) = &texture_bind_group {
                rpass.set_bind_group(1, &texture_bind_group, &[]);
            }

            rpass.draw(0..n_vertices as u32, 0..1);
        }
    }
}

impl<S: BaseStimulusImplementation> BaseStimulus<S> {
    /// Create a new stimulus.
    pub fn create(
        window: &Window,
        window_state: &WindowState,
        implementation: S,
    ) -> Self {
        let device = &window_state.device;
        let queue = &window_state.queue;
        let surface = &window_state.surface;
        let adapter = &window_state.adapter;
        let sconfig = window_state.config.clone();

        // get the geometry
        let shape = implementation.get_geometry();
        // get the texture size
        let texture_size = implementation.get_texture_size();
        // get the fragment shader code
        let fragment_shader_code = implementation.get_fragment_shader_code();
        // get the uniform buffer data
        let params_bytes = implementation
            .get_uniform_buffer_data()
            .unwrap_or(&[0u8; 2]);

        // compile the fragment shader
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                (include_str!("shaders/base.wgsl").to_string() + &fragment_shader_code)
                    .into(),
            ),
        });

        // compile the vertex shader
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/base.wgsl").into()),
        });

        let width_mm = window.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm = window.viewing_distance.load(Ordering::Relaxed);
        let width_px = sconfig.width;
        let height_px = sconfig.height;

        // create the vertex buffer
        let vertices =
            shape.to_vertices_px(width_mm, viewing_distance_mm, width_px, height_px);
        let n_vertices = vertices.len();
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        // create the uniform buffer #1
        let stimulus_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stimulus Buffer"),
                contents: params_bytes,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // create the uniform buffer #2 (for the transformation matrix
        let transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(
                    &nalgebra::Matrix4::<f32>::identity().as_slice(),
                ),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // uniform buffer #1 (contains the stimulus parameters)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // uniform buffer #2
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: stimulus_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: transform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // if a texture size is specified, create a texture
        let texture = if let Some(texture_size) = texture_size {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // TODO: this should be configurable
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[wgpu::TextureFormat::Bgra8Unorm], // allow reading texture in linear space
            });
            Some(texture)
        } else {
            None
        };

        // if a texture is specified, create a sampler and bind group
        let (texture_bind_group_layout, texture_bind_group) = if let Some(texture) =
            &texture
        {
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                ..Default::default()
            });
            let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let texture_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(
                                wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

            let texture_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture_sampler),
                        },
                    ],
                    label: Some("texture_bind_group"),
                });

            (Some(texture_bind_group_layout), Some(texture_bind_group))
        } else {
            (None, None)
        };

        // create the bind group layout (depending on whether a texture is specified)
        let bind_group_layouts =
            if let Some(texture_bind_group_layout) = &texture_bind_group_layout {
                vec![&uniform_bind_group_layout, &texture_bind_group_layout]
            } else {
                vec![&uniform_bind_group_layout]
            };

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: bind_group_layouts.as_slice(),
                push_constant_ranges: &[],
            });

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = TextureFormat::Bgra8Unorm;

        log::warn!("swapchain format: {:?}", swapchain_capabilities.formats);

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: "fs_main",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let oot = Self {
            uniform_buffer: Arc::new(Mutex::new(stimulus_buffer)),
            bind_group: Arc::new(Mutex::new(uniform_bind_group)),
            pipeline: Arc::new(Mutex::new(render_pipeline)),
            window: window.clone(),
            stimulus_implementation: Arc::new(Mutex::new(implementation)),
            transforms: Arc::new(Mutex::new(Transformation2D::Identity)),
            vertex_buffer: Arc::new(Mutex::new(vertex_buffer)),
            transform_buffer: Arc::new(Mutex::new(transform_buffer)),
            n_vertices: AtomicUsize::new(n_vertices),
            texture_size: if let Some(texture_size) = texture_size {
                Some(texture_size)
            } else {
                None
            },
            texture: if let Some(texture) = texture {
                Some(Arc::new(Mutex::new(texture)))
            } else {
                None
            },
            texture_bind_group: if let Some(texture_bind_group) = texture_bind_group {
                Some(Arc::new(Mutex::new(texture_bind_group)))
            } else {
                None
            },
        };

        // if a texture is specified, upload the texture data
        if let Some(texture) = &oot.texture {
            let texture = texture.lock_blocking();
            let texture_data = oot
                .stimulus_implementation
                .lock_blocking()
                .get_texture_data()
                .unwrap();
            oot.set_texture(texture_data.as_slice(), &queue, &texture);
        }

        oot
    }

    pub fn set_transformation(&mut self, transform: Transformation2D) -> &mut Self {
        *(self.transforms.lock_blocking()) = transform;
        self
    }

    /// Set the data for the texture of the stimulus.
    ///
    /// # Performance considerations
    /// While there should be no problem in calling this method multiple times per frame, it is not recommended to do so as it will
    /// cause the texture to be re-uploaded to the GPU multiple times, as every call to this method will result in a call to `queue.write_texture()`.
    ///
    /// # Arguments
    ///
    /// * `data` - The data for the texture. The data must be a slice of `f16` values. The length of the slice must match the size of the texture.
    ///
    /// # Panics
    ///
    /// This method will panic if the data size does not match the texture size or if no texture is specified for this stimulus.
    pub fn set_texture(&self, data: &[u8], queue: &Queue, texture: &wgpu::Texture) {
        // get a view of u8 from the f16 data using bytemuck
        // let data: &[u8] = bytemuck::cast_slice(data);

        let width = texture.size().width;
        let height = texture.size().height;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
