use crate::utils::AtomicExt;
use crate::visual::geometry::Vertex;
use crate::visual::window::WindowState;
use crate::visual::{
    geometry::{ToVertices, Transformation2D},
    Window,
};
use crate::GPUState;
use async_lock::Mutex;
use std::sync::{atomic::AtomicUsize, Arc};
use wgpu::util::DeviceExt;
use wgpu::TextureFormat;
use winit::window;

use super::Stimulus;

/// Base stimulus that serves as a template for almost all stimuli.
#[derive(Clone)]
pub struct BaseStimulus {
    /// The window used to create the stimulus.
    window: Window,
    /// The rendering pipeline for the stimulus.
    pipeline: Arc<Mutex<wgpu::RenderPipeline>>,
    /// The geometry of the stimulus.
    geometry: Arc<Mutex<Box<dyn ToVertices>>>,
    /// A `Transformation2D` that will be applied in the vertex shader.
    transforms: Arc<Mutex<Transformation2D>>,
    /// Vertex buffer that will be uploaded to the shader.
    vertex_buffer: Arc<Mutex<wgpu::Buffer>>,
    /// Number of vertices.
    n_vertices: Arc<AtomicUsize>,
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

// manually implement Debug for BaseStimulus
impl std::fmt::Debug for BaseStimulus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseStimulus")
            .field("window", &self.window)
            .field("pipeline", &self.pipeline)
            .field("transforms", &self.transforms)
            .field("vertex_buffer", &self.vertex_buffer)
            .field("n_vertices", &self.n_vertices)
            .field("bind_group", &self.bind_group)
            .field("uniform_buffer", &self.uniform_buffer)
            .field("transform_buffer", &self.transform_buffer)
            .field("texture_size", &self.texture_size)
            .field("texture", &self.texture)
            .field("texture_bind_group", &self.texture_bind_group)
            .finish()
    }
}

impl BaseStimulus {
    pub fn new(
        window: &Window,
        geometry: impl ToVertices + 'static,
        fragment_shader_code: &str,
        texture_size: Option<wgpu::Extent3d>,
        uniform_buffer_data: Option<&[u8]>,
    ) -> Self {
        // get the GPU state
        let gpu_state = window.read_gpu_state_blocking();
        let window_state = window.read_window_state_blocking();
        let device = &gpu_state.device;
        let surface = &window_state.surface;
        let adapter = &gpu_state.adapter;
        let surface_config = window_state.config.clone();

        // get the uniform buffer data
        let uniform_buffer_data = uniform_buffer_data.unwrap_or(&[0u8; 2]);

        // compile the vertex shader
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/base.wgsl").into()),
        });

        // compile the fragment shader
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                (include_str!("shaders/base.wgsl").to_string() + &fragment_shader_code)
                    .into(),
            ),
        });

        let width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let width_px = surface_config.width;
        let height_px = surface_config.height;

        // create the vertex buffer
        let vertices =
            geometry.to_vertices_px(width_mm, viewing_distance_mm, width_px, height_px);
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
                contents: uniform_buffer_data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // create the uniform buffer #2 (note that to avoid memort alignment issues, we use a 4x4 matrix,
        // even though we only need a 3x3 matrix for the 2D transformation)
        // TODO: we should proably fix this in the future
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

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = TextureFormat::Bgra8Unorm;

        // // if a texture is specified, upload the texture data
        // if let Some(texture) = &oot.texture {
        //     let texture = texture.lock_blocking();
        //     let texture_data = oot
        //         .stimulus_implementation
        //         .lock_blocking()
        //         .get_texture_data()
        //         .unwrap();
        //     oot.set_texture(texture_data.as_slice(), &queue, &texture);
        // }

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

        Self {
            window: window.clone(),
            geometry: Arc::new(Mutex::new(Box::new(geometry))),
            uniform_buffer: Arc::new(Mutex::new(stimulus_buffer)),
            bind_group: Arc::new(Mutex::new(uniform_bind_group)),
            pipeline: Arc::new(Mutex::new(render_pipeline)),
            transforms: Arc::new(Mutex::new(Transformation2D::Identity)),
            vertex_buffer: Arc::new(Mutex::new(vertex_buffer)),
            transform_buffer: Arc::new(Mutex::new(transform_buffer)),
            n_vertices: Arc::new(AtomicUsize::new(n_vertices)),
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
        }
    }

    pub fn set_transformation(&self, transform: Transformation2D) {
        *self.transforms.lock_blocking() = transform;
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
    /// If no texture is specified, this method is a no-op.
    pub fn set_texture(&self, data: &[u8]) {
        // get the GPU state
        let gpu_state = self.window.read_gpu_state_blocking();
        let queue = &gpu_state.queue;

        if let Some(texture) = &self.texture {
            let texture = texture.lock_blocking();

            // get the texture size
            let width = texture.size().width;
            let height = texture.size().height;

            // upload the texture data
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

    pub fn set_uniform_buffer(&self, data: &[u8], gpu_state: &GPUState) {
        let uniform_buffer = self.uniform_buffer.lock_blocking();
        gpu_state.queue.write_buffer(&uniform_buffer, 0, data);
    }

    pub fn set_geometry(&self, geometry: impl ToVertices + 'static) {
        self.n_vertices.store_relaxed(geometry.n_vertices());
        *self.geometry.lock_blocking() =
            Box::new(geometry) as Box<dyn ToVertices + 'static>;
        // we dont upload the vertex buffer here, as this will need to be done every time the frame is rendered
    }
}

impl Stimulus for BaseStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        let screen_width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();

        let screen_width_px = window_state.config.width;
        let screen_height_px = window_state.config.height;

        let geometry = self.geometry.lock_blocking();

        let vertices = geometry.to_vertices_px(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        // update the vertex buffer
        gpu_state.queue.write_buffer(
            &(self.vertex_buffer.lock_blocking()),
            0,
            bytemuck::cast_slice(&vertices),
        );

        // update the transform buffer
        let win_transform =
            Window::transformation_matrix_to_ndc(screen_width_px, screen_height_px)
                .map(|x| x as f32);

        // then get the transformation matrix from the stimulus
        let stim_transform = self.transforms.lock_blocking().to_transformation_matrix(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        );

        // multiply the two matrices
        let transform = win_transform * stim_transform.transpose();

        // add the 4th row and column (for memory alignment)
        let transform = transform.to_homogeneous();

        gpu_state.queue.write_buffer(
            &(self.transform_buffer.lock_blocking()),
            0,
            bytemuck::cast_slice(transform.as_slice()),
        );
    }

    fn render(&mut self, enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> () {
        let pipeline = self.pipeline.lock_blocking();
        let n_vertices = self.n_vertices.load_relaxed();

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
