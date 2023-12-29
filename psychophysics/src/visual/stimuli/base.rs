use crate::utils::BlockingLock;
use crate::visual::geometry::Size;
use async_lock::Mutex;
use half::f16;
use ndarray::ArrayView;
use rodio::queue;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wgpu::TextureFormat;

use crate::visual::Renderable;
use bytemuck::{Pod, Zeroable};

use super::super::geometry::ToVertices;
use super::super::geometry::Transformation2D;
use super::super::geometry::Vertex;
use super::super::pwindow::Window;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, ShaderModule, SurfaceConfiguration};

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
    /// * `Option<P>` - If the stimulus has changed, this should return `Some(params)`, otherwise `None`.
    /// * `Option<Box<dyn ToVertices>>` - If the stimulus has changed, this should return `Some(geometry)`, otherwise `None`.
    /// * `Option<Vec<u8>>` - If the stimulus has changed, this should return `Some(texture_data)`, otherwise `None`.
    fn update(
        &mut self,
        screen_width_mm: f64,
        viewing_distance_mm: f64,
        screen_width_px: i32,
        screen_height_px: i32,
    ) -> (Option<P>, Option<Box<dyn ToVertices>>, Option<Vec<u8>>) {
        (None, None, None)
    }
    /// Returns the the function that will be used as the fragment shader.
    fn get_fragment_shader_code(&self) -> String;

    /// Returns the uniform buffer byte slice that will be used to initialize the uniform buffer.
    /// If no uniform parameters are needed, this should return `None` (default).
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

/// A very flexible stimulus that can be paramterized with a custom shader and custom geometry.
/// Takes care of many important aspects of stimulus rendering like setting-up the rednering
/// pipeline, updating vertex, uniform, and texture buffers, applying transformations and more.
pub struct BaseStimulus<S: BaseStimulusImplementation> {
    /// The implementation of the stimulus.
    pub(crate) stimulus_implementation: Arc<Mutex<S>>,
    /// The geometry, must implement the `ToVertex` trait.
    geometry: Arc<Mutex<G>>,
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
unsafe impl<
        G: ToVertices,
        S: BaseStimulusImplementation<P>,
        P: BaseStimulusParams,
    > Send for BaseStimulus<G, S, P>
{
}
// same here
unsafe impl<
        G: ToVertices,
        S: BaseStimulusImplementation<P>,
        P: BaseStimulusParams,
    > Sync for BaseStimulus<G, S, P>
{
}

impl<
        G: ToVertices,
        S: BaseStimulusImplementation<P>,
        P: BaseStimulusParams,
    > Clone for BaseStimulus<G, S, P>
{
    fn clone(&self) -> Self {
        Self {
            uniform_buffer: self.uniform_buffer.clone(),
            bind_group: self.bind_group.clone(),
            pipeline: self.pipeline.clone(),
            window: self.window.clone(),
            stimulus_implementation: self.stimulus_implementation.clone(),
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

impl<
        G: ToVertices,
        S: BaseStimulusImplementation<P>,
        P: BaseStimulusParams,
    > Renderable for BaseStimulus<G, S, P>
{
    fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _view: &wgpu::TextureView,
        config: &SurfaceConfiguration,
        window_handle: &super::super::pwindow::Window,
    ) -> () {
        let screen_width_mm =
            window_handle.physical_width.load(Ordering::Relaxed);
        let viewing_distance_mm =
            window_handle.viewing_distance.load(Ordering::Relaxed);
        let screen_width_px = config.width as i32;
        let screen_height_px = config.height as i32;

        // call the shader update function
        let (params, tex_data) =
            self.stimulus_implementation.lock_blocking().update(
                &mut (self.stimulus_implementation_params.lock_blocking()),
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            );

        // if the shader returned new texture data, update the texture
        if let Some(tex_data) = tex_data {
            let binding = self.texture.clone().unwrap();
            let texture = binding.lock_blocking();
            self.set_texture(tex_data.as_slice(), &queue, &texture);
        }

        // if the shader returned new parameters, update the uniform buffer
        if let Some(params) = params {
            queue.write_buffer(
                &(self.uniform_buffer.lock_blocking()),
                0,
                bytemuck::cast_slice(&[params]),
            );
        }

        let geometry = (self.geometry.lock_blocking());

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
            &(self.vertex_buffer.lock_blocking()),
            0,
            bytemuck::cast_slice(&vertices),
        );

        // update the transform buffer
        // first get the transformation matrix from the window handle
        let win_transform = Window::transformation_matrix_to_ndc(
            screen_width_px,
            screen_height_px,
        )
        .map(|x| x as f32);
        let stim_transform = (self.transforms.lock_blocking())
            .to_transformation_matrix(
                screen_width_mm,
                viewing_distance_mm,
                screen_width_px,
                screen_height_px,
            );

        // cast to f32
        let transform = (win_transform * stim_transform);

        queue.write_buffer(
            &(self.transform_buffer.lock_blocking()),
            0,
            bytemuck::cast_slice(transform.as_slice()),
        );
    }

    fn render(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> () {
        let pipeline = (self.pipeline.lock_blocking());
        let n_vertices = self.n_vertices.load(Ordering::Relaxed);
        let texture_bind_group =
            if let Some(texture_bind_group) = &self.texture_bind_group {
                Some((texture_bind_group.lock_blocking()))
            } else {
                None
            };

        let bind_group = (self.bind_group.lock_blocking());
        let vertex_buffer = (self.vertex_buffer.lock_blocking());
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

impl<
        G: ToVertices,
        S: BaseStimulusImplementation<P>,
        P: BaseStimulusParams,
    > BaseStimulus<G, S, P>
{
    /// Create a new stimulus.
    pub fn create(window_handle: &Window, implementation: S) -> Self {
        let window = (window_handle.get_window_state_blocking());
        let device = &window.device;
        let surface = &window.surface;
        let adapter = &window.adapter;
        let sconfig = window.config.clone();

        // get the geometry
        let shape = implementation.get_geometry();
        // get the texture size
        let texture_size = implementation.get_texture_size();
        // get the fragment shader code
        let fragment_shader_code = implementation.get_fragment_shader_code();

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
                contents: stim_params.as_byteslice(),
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
        let (texture_bind_group_layout, texture_bind_group) =
            if let Some(texture) = &texture {
                let texture_view =
                    texture.create_view(&wgpu::TextureViewDescriptor {
                        format: Some(wgpu::TextureFormat::Bgra8Unorm),
                        ..Default::default()
                    });
                let texture_sampler =
                    device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Nearest,
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
        let swapchain_format = TextureFormat::Bgra8Unorm;

        log::warn!("swapchain format: {:?}", swapchain_capabilities.formats);

        // first, we create the shader module
        let shader_module =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    implementation
                        .get_vertex_shader()
                        .unwrap_or(&String::from(include_str!(
                            "../../shaders/default_vertex.wgsl"
                        )))
                        .into(),
                ),
            });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_module,
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
            stimulus_implementation: Arc::new(Mutex::new(implementation)),
            stimulus_implementation_params: Arc::new(Mutex::new(stim_params)),
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

    pub fn set_transformation(
        &mut self,
        transform: Transformation2D,
    ) -> &mut Self {
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
    pub fn set_texture(
        &self,
        data: &[u8],
        queue: &Queue,
        texture: &wgpu::Texture,
    ) {
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
