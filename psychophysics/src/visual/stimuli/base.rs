use async_lock::Mutex;
use futures_lite::future::block_on;
use ndarray::ArrayView;
use rodio::queue;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::visual::Renderable;
use bytemuck::{Pod, Zeroable};

use super::super::geometry::ToVertices;
use super::super::geometry::Transformation2D;
use super::super::geometry::Vertex;
use super::super::pwindow::WindowHandle;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, ShaderModule, SurfaceConfiguration};

pub trait BaseStimulusPixelShader<P: ShapeStimulusParams> {
    /// Called when `prepare()` is called on the stimulus.
    fn prepare(
        &self,
        params: &mut P,
        screen_width_mm: f64,
        viewing_distance_mm: f64,
        screen_width_px: i32,
        screen_height_px: i32,
    ) -> ();
    fn get_shader(&self) -> &ShaderModule;
}
pub trait ShapeStimulusParams: Pod + Zeroable + Copy {}

/// A very flexible stimulus that can be paramterized with a custom shader and custom geometry.
/// Takes care of many important aspects of stimulus rendering like setting-up the rednering
/// pipeline, updating vertex, uniform, and texture buffers, applying transformations and more.
pub struct BaseStimulus<
    G: ToVertices, // a type that can be converted to vertices
    S: BaseStimulusPixelShader<P>, // a type that implements the BaseStimulusPixelShader trait
    P: ShapeStimulusParams, // a type that implements the ShapeStimulusParams trait
> {
    /// The shader
    pub(crate) pixel_shader: Arc<Mutex<S>>,
    /// The paramters that are send to the shader through a uniform buffer.
    pub(crate) pixel_shader_params: Arc<Mutex<P>>,
    /// The geometry, must implement the `ToVertex` trait.
    geometry: Arc<Mutex<G>>,
    /// The rendering pipeline for the stimulus.
    pipeline: Arc<Mutex<wgpu::RenderPipeline>>,
    /// The window used to create the stimulus.
    window: WindowHandle,
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
    texture: Option<Arc<Mutex<wgpu::Texture>>>,
    /// (Optional) texture bind group.
    texture_bind_group: Option<Arc<Mutex<wgpu::BindGroup>>>,
}

impl<G: ToVertices, S: BaseStimulusPixelShader<P>, P: ShapeStimulusParams> Clone
    for BaseStimulus<G, S, P>
{
    fn clone(&self) -> Self {
        Self {
            uniform_buffer: self.uniform_buffer.clone(),
            bind_group: self.bind_group.clone(),
            pipeline: self.pipeline.clone(),
            window: self.window.clone(),
            pixel_shader_params: self.pixel_shader_params.clone(),
            pixel_shader: self.pixel_shader.clone(),
            geometry: self.geometry.clone(),
            transforms: self.transforms.clone(),
            vertex_buffer: self.vertex_buffer.clone(),
            transform_buffer: self.transform_buffer.clone(),
            n_vertices: AtomicUsize::new(
                self.n_vertices.load(Ordering::Relaxed),
            ),
            texture_size: self.texture_size.clone(),
            texture: self.texture.clone(),
            texture_bind_group: self.texture_bind_group.clone(),
        }
    }
}

impl<G: ToVertices, S: BaseStimulusPixelShader<P>, P: ShapeStimulusParams>
    Renderable for BaseStimulus<G, S, P>
{
    fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &super::super::pwindow::WindowHandle,
    ) -> () {
        let screen_width_mm =
            window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = config.width as i32;
        let screen_height_px = config.height as i32;

        // call the shader update function
        // TODO: does this need to be a blocking call?
        block_on(self.pixel_shader.lock()).prepare(
            &mut (block_on(self.pixel_shader_params.lock())),
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );
        let geometry = block_on(self.geometry.lock());
        // get vertices from geometry
        let vertices = geometry.to_vertices_px(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );
        self.n_vertices.store(vertices.len(), Ordering::Relaxed);

        // update the vertex buffer
        queue.write_buffer(
            &block_on(self.vertex_buffer.lock()),
            0,
            bytemuck::cast_slice(&vertices),
        );

        // update the stimulus buffer
        queue.write_buffer(
            &block_on(self.uniform_buffer.lock()),
            0,
            bytemuck::cast_slice(&[*block_on(self.pixel_shader_params.lock())]),
        );

        // update the transform buffer
        // first get the transformation matrix from the window handle
        let win_transform = WindowHandle::transformation_matrix_to_ndc(
            screen_width_px,
            screen_height_px,
        )
        .map(|x| x as f32);
        let stim_transform = block_on(self.transforms.lock())
            .to_transformation_matrix(
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            );
        // cast to f32
        let transform = (win_transform * stim_transform);
        queue.write_buffer(
            &block_on(self.transform_buffer.lock()),
            0,
            bytemuck::cast_slice(transform.as_slice()),
        );
    }

    fn render(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> () {
        let pipeline = block_on(self.pipeline.lock());
        let n_vertices = self.n_vertices.load(Ordering::Relaxed);
        let texture_bind_group =
            if let Some(texture_bind_group) = &self.texture_bind_group {
                Some(block_on(texture_bind_group.lock()))
            } else {
                None
            };

        let bind_group = block_on(self.bind_group.lock());
        let vertex_buffer = block_on(self.vertex_buffer.lock());
        {
            let mut rpass =
                enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
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

impl<G: ToVertices, S: BaseStimulusPixelShader<P>, P: ShapeStimulusParams>
    BaseStimulus<G, S, P>
{
    /// Create a new stimulus.
    pub fn create(
        window_handle: &WindowHandle,
        shader: S,
        shape: G,
        stim_params: P,
        texture_size: Option<wgpu::Extent3d>,
    ) -> Self {
        let window = block_on(window_handle.get_window());
        let device = &window.device;
        let surface = &window.surface;
        let adapter = &window.adapter;
        let sconfig = window.config.clone();

        let width_mm = window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window_handle.viewing_distance.load(Ordering::Relaxed);
        let width_px = sconfig.width as i32;
        let height_px = sconfig.height as i32;

        // create the vertex buffer
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(
                    shape
                        .to_vertices_px(
                            width_mm,
                            viewing_distance_mm,
                            width_px,
                            height_px,
                        )
                        .as_slice(),
                ),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
            });

        // create the uniform buffer #1
        let stimulus_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stimulus Buffer"),
                contents: bytemuck::cast_slice(&[stim_params]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        // create the uniform buffer #2 (for the transformation matrix
        let transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(
                    &nalgebra::Matrix4::<f32>::identity().as_slice(),
                ),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // uniform buffer #1
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

        let uniform_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            });
            Some(texture)
        } else {
            None
        };

        // if a texture is specified, create a sampler and bind group
        let (texture_bind_group_layout, texture_bind_group) =
            if let Some(texture) = &texture {
                let texture_view = texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let texture_sampler =
                    device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    });

                let texture_bind_group_layout = device
                    .create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        multisampled: false,
                                        view_dimension:
                                            wgpu::TextureViewDimension::D2,
                                        sample_type:
                                            wgpu::TextureSampleType::Float {
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
                        },
                    );

                let texture_bind_group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &texture_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &texture_view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(
                                    &texture_sampler,
                                ),
                            },
                        ],
                        label: Some("texture_bind_group"),
                    });

                (Some(texture_bind_group_layout), Some(texture_bind_group))
            } else {
                (None, None)
            };

        // create the bind group layout (depending on whether a texture is specified)
        let bind_group_layouts = if let Some(texture_bind_group_layout) =
            &texture_bind_group_layout
        {
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
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader.get_shader(),
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader.get_shader(),
                    entry_point: "fs_main",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        Self {
            uniform_buffer: Arc::new(Mutex::new(stimulus_buffer)),
            bind_group: Arc::new(Mutex::new(uniform_bind_group)),
            pipeline: Arc::new(Mutex::new(render_pipeline)),
            window: window_handle.clone(),
            pixel_shader: Arc::new(Mutex::new(shader)),
            pixel_shader_params: Arc::new(Mutex::new(stim_params)),
            geometry: Arc::new(Mutex::new(shape)),
            transforms: Arc::new(Mutex::new(Transformation2D::Identity)),
            vertex_buffer: Arc::new(Mutex::new(vertex_buffer)),
            transform_buffer: Arc::new(Mutex::new(transform_buffer)),
            n_vertices: AtomicUsize::new(0), // will be updated in prepare
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
            texture_bind_group: if let Some(texture_bind_group) =
                texture_bind_group
            {
                Some(Arc::new(Mutex::new(texture_bind_group)))
            } else {
                None
            },
        }
    }

    pub fn set_transformation(&self, transform: Transformation2D) {
        *block_on(self.transforms.lock()) = transform;
    }

    /// Set the data for the texture of the stimulus.
    ///
    /// # Performance considerations
    /// While there should be no problem in calling this method multiple times per frame, it is not recommended to do so as it will
    /// cause the texture to be re-uploaded to the GPU multiple times, as every call to this method will result in a call to `queue.write_texture()`.
    ///
    /// # Arguments
    ///
    /// * `data` - The data for the texture. Must be a 2D array of RGBA values matching the size of the texture. The data is expected to be in row-major order.
    ///
    /// # Panics
    ///
    /// This method will panic if the data size does not match the texture size or if no texture is specified for this stimulus.
    pub fn set_texture(&self, data: &[u8]) {
        if let Some(texture) = &self.texture {
            let texture = block_on(texture.lock());
            let width = texture.size().width;
            let height = texture.size().height;
            // check if the data has the correct size
            if data.len() != (4 * width * height) as usize {
                panic!("Data size does not match texture size. Expected {} bytes (texture size) but got {} bytes (data size).", (4 * width * height) as usize, data.len());
            }
            let queue = &block_on(self.window.get_window()).queue;

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
        } else {
            panic!("No texture specified for this stimulus.");
        }
    }
}
