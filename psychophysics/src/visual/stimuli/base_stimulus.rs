use crate::utils::AtomicExt;
use crate::visual::geometry::Vertex;
use crate::visual::window::WindowState;
use crate::visual::{
    geometry::{ToVertices, Transformation2D},
    Window,
};
use crate::GPUState;
use async_lock::Mutex;
use num_traits::ToPrimitive;
use std::sync::atomic::AtomicBool;
use std::sync::{atomic::AtomicUsize, Arc};
use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use super::Stimulus;

// helper trait to set the texture data
pub trait TextureDataTrait {
    fn to_f32(self) -> Vec<f32>;
    fn to_f16(self) -> Vec<half::f16>;
    fn to_bytes(self) -> Vec<u8>;
}
// implement the trait for Vec<f32>
impl TextureDataTrait for Vec<f32> {
    fn to_f32(self) -> Vec<f32> {
        self
    }
    fn to_f16(self) -> Vec<half::f16> {
        self.iter().map(|x| half::f16::from_f32(*x)).collect()
    }
    fn to_bytes(self) -> Vec<u8> {
        self.iter().map(|x| x.to_le_bytes()).flatten().collect()
    }
}
// implement the trait for Vec<half::f16>
impl TextureDataTrait for Vec<half::f16> {
    fn to_f32(self) -> Vec<f32> {
        self.iter().map(|x| x.to_f32()).collect()
    }
    fn to_f16(self) -> Vec<half::f16> {
        self
    }
    fn to_bytes(self) -> Vec<u8> {
        self.iter().map(|x| x.to_le_bytes()).flatten().collect()
    }
}
// implement the trait for Vec<u8>
impl TextureDataTrait for Vec<u8> {
    fn to_f32(self) -> Vec<f32> {
        self.iter().map(|x| (*x as f32) / 255.0).collect()
    }
    fn to_f16(self) -> Vec<half::f16> {
        self.to_f32().to_f16() // convert to f32 first to allow for higher precision division
    }
    fn to_bytes(self) -> Vec<u8> {
        self
    }
}

const VERTEX_SHADER: &str = "
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};


@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;
@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    // transform the vertex position
    let transform3x3 = mat3x3<f32>(transform[0].xyz, transform[1].xyz, transform[2].xyz);
    let new_position = transform3x3 * vec3(model.position.xy, 1.0);

    return VertexOutput(
        vec4(new_position, 1.0),
        vec2<f32>(model.position.xy),
        model.tex_coords
    );
}";

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
    /// Bind group 0 (contains the transformation matrix and, if a texture is specified, the texture and sampler).
    tts_bind_group: Arc<Mutex<wgpu::BindGroup>>,
    /// Bind group 1 (contains the uniform tts_bind_groupbuffers).
    uniform_bind_group: Arc<Mutex<wgpu::BindGroup>>,
    /// Uniform buffer for the pixel shader paramters.
    uniform_buffers: Arc<Mutex<Vec<wgpu::Buffer>>>,
    /// Unifrom buffer for the transformation matrix.
    transform_buffer: Arc<Mutex<wgpu::Buffer>>,
    /// (Optional) texture size.
    texture_size: Option<wgpu::Extent3d>,
    /// (Optional) texture.
    texture: Option<Arc<Mutex<wgpu::Texture>>>,
    /// Flag that indicates if the stimulus is visible or not.
    visible: Arc<AtomicBool>,
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
            .field("bind_group", &self.uniform_bind_group)
            .field("uniform_buffers", &self.uniform_buffers)
            .field("transform_buffer", &self.transform_buffer)
            .field("texture_size", &self.texture_size)
            .field("texture", &self.texture)
            .field("texture_bind_group", &self.tts_bind_group)
            .finish()
    }
}

impl BaseStimulus {
    pub fn new<T>(
        window: &Window,
        geometry: impl ToVertices + 'static,
        fragment_shader_code: &str,
        texture_size: Option<wgpu::Extent3d>,
        texture_data: Option<T>,
        uniform_buffers_data: &[Vec<u8>],
    ) -> Self
    where
        T: TextureDataTrait,
    {
        // get the GPU state
        let gpu_state = window.read_gpu_state_blocking();
        let window_state = window.read_window_state_blocking();
        let device = &gpu_state.device;
        let surface_config = window_state.config.clone();

        // iter over uniform buffer data and create a buffer each
        // all buffers will be part ofnthe same bind group (bind group 1)
        let mut uniform_buffer_bind_group_layout_entries = vec![];
        let mut uniforms_buffers = vec![];

        for (i, uniform_buffer_data) in uniform_buffers_data.iter().cloned().enumerate() {
            // create bind group layout entry
            let uniform_buffer_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };

            // add the entry to the list of entries
            uniform_buffer_bind_group_layout_entries
                .push(uniform_buffer_bind_group_layout_entry);

            // create the buffer
            let uniform_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: uniform_buffer_data.as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

            // add the buffer to the list of buffers
            uniforms_buffers.push(uniform_buffer);
        }

        // create uniform_buffers_bind_group_entries by iterating over the uniform_buffers
        let uniform_buffers_bind_group_entries = uniforms_buffers
            .iter()
            .enumerate()
            .map(|(i, uniform_buffer)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: uniform_buffer.as_entire_binding(),
            })
            .collect::<Vec<_>>();

        //     // create the bind group entry
        //     let uniform_buffer_bind_group_entry = wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: uniform_buffer.as_entire_binding(),
        //     };

        //     // add the entry to the list of entries
        //     uniform_buffers_bind_group_entries.push(uniform_buffer_bind_group_entry);
        // }

        let uniform_buffer_bind_group_layout_entries =
            uniform_buffer_bind_group_layout_entries.as_slice();

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: uniform_buffer_bind_group_layout_entries,
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group_entries = uniform_buffers_bind_group_entries.as_slice();

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: uniform_bind_group_entries,
            label: Some("uniform_bind_group"),
        });

        let transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(
                    &nalgebra::Matrix4::<f32>::identity().as_slice(),
                ),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // if a texture size is specified, create a texture
        let texture = if let Some(texture_size) = texture_size {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                label: Some("some texture"),
                view_formats: &[wgpu::TextureFormat::Bgra8Unorm], // allow reading texture in linear space
            });
            Some(texture)
        } else {
            None
        };

        // create bind group layout for bind group 0
        // this bind group will contain the transformation matrix
        // and, if a texture is specified, the texture + sampler (hence: tts)
        let mut tts_bind_bind_group_layout_entries = vec![];
        let mut tts_bind_group_entries = vec![];

        // push the transformation matrix entry
        tts_bind_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        tts_bind_group_entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        });

        // if a texture is specified, create a texture buffer and a sampler and add them to the bind group
        let (tts_bind_bind_group_layout, tts_bind_group) = if let Some(ref texture) =
            texture
        {
            // create the texture view
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                ..Default::default()
            });

            // create the sampler
            let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let view_dimension = {
                if texture.depth_or_array_layers() > 1 {
                    wgpu::TextureViewDimension::D2Array
                } else {
                    wgpu::TextureViewDimension::D2
                }
            };

            // add the texture view to the bind group
            tts_bind_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: view_dimension,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });

            // add the sampler to the bind group
            tts_bind_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });

            tts_bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            });

            tts_bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&texture_sampler),
            });

            // create the bind group layout for bind group 0
            let tts_bind_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: tts_bind_bind_group_layout_entries.as_slice(),
                    label: Some("tts_bind_bind_group_layout"),
                });

            // create the bind group for bind group 0
            let tts_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &tts_bind_bind_group_layout,
                entries: tts_bind_group_entries.as_slice(),
                label: Some("tts_bind_group"),
            });

            (tts_bind_bind_group_layout, tts_bind_group)
        } else {
            // create the bind group layout for bind group 0
            let tts_bind_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: tts_bind_bind_group_layout_entries.as_slice(),
                    label: Some("tts_bind_bind_group_layout"),
                });

            // create the bind group for bind group 0
            let tts_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &tts_bind_bind_group_layout,
                entries: tts_bind_group_entries.as_slice(),
                label: Some("tts_bind_group"),
            });

            (tts_bind_bind_group_layout, tts_bind_group)
        };

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &tts_bind_bind_group_layout,
                    &uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let swapchain_format = TextureFormat::Bgra8Unorm;

        // if a texture is specified, upload the texture data

        // compile the vertex shader
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });

        // compile the fragment shader
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(fragment_shader_code.into()),
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
                    targets: &[Some(wgpu::ColorTargetState {
                        format: swapchain_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        // upload the texture data if a texture is specified

        let out = Self {
            window: window.clone(),
            geometry: Arc::new(Mutex::new(Box::new(geometry))),
            uniform_buffers: Arc::new(Mutex::new(uniforms_buffers)),
            uniform_bind_group: Arc::new(Mutex::new(uniform_bind_group)),
            pipeline: Arc::new(Mutex::new(render_pipeline)),
            transforms: Arc::new(Mutex::new(Transformation2D::Identity)),
            vertex_buffer: Arc::new(Mutex::new(vertex_buffer)),
            transform_buffer: Arc::new(Mutex::new(transform_buffer)),
            n_vertices: Arc::new(AtomicUsize::new(n_vertices)),
            texture_size: texture_size,
            texture: if let Some(texture) = texture {
                Some(Arc::new(Mutex::new(texture)))
            } else {
                None
            },
            tts_bind_group: Arc::new(Mutex::new(tts_bind_group)),
            visible: Arc::new(AtomicBool::new(true)),
        };

        // if a texture is specified, upload the texture data
        if let Some(texture_data) = texture_data {
            out.set_texture(texture_data, &gpu_state);
        }

        out
    }

    /// Set the visibility of the stimulus.
    pub fn set_visible(&self, visible: bool) {
        self.visible.store_relaxed(visible);
    }

    /// Set the data for the texture of the stimulus.
    ///
    /// # Performance considerations
    /// While there should be no problem in calling this method multiple times per frame, it is not recommended to do so as it will
    /// cause the texture to be re-uploaded to the GPU multiple times, as every call to this method will result in a call to `queue.write_texture()`.
    ///
    /// # Arguments
    ///
    /// * `data` - The data for the texture. The data must be a slice of `f32` values. The length of the slice must match the size of the texture.
    ///
    /// If no texture is specified, this method is a no-op.
    pub fn set_texture<T>(&self, data: T, gpu_state: &GPUState)
    where
        T: TextureDataTrait,
    {
        // convert to bytes
        let data = data.to_bytes();

        // get the GPU state
        let queue = &gpu_state.queue;

        if let Some(texture) = &self.texture {
            let texture = texture.lock_blocking();

            // upload the texture data
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data.as_slice(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 1 * texture.size().width),
                    rows_per_image: Some(texture.size().height),
                },
                texture.size(),
            );
        }
    }

    pub fn set_uniform_buffers(&self, data: &[&[u8]], gpu_state: &GPUState) {
        let queue = &gpu_state.queue;
        let uniform_buffers = self.uniform_buffers.lock_blocking();

        for (i, buffer) in uniform_buffers.iter().enumerate() {
            queue.write_buffer(buffer, 0, data[i]);
        }
    }

    pub fn set_geometry(&self, geometry: impl ToVertices + 'static) {
        self.n_vertices.store_relaxed(geometry.n_vertices());
        *self.geometry.lock_blocking() =
            Box::new(geometry) as Box<dyn ToVertices + 'static>;
        // we dont upload the vertex buffer here, as this will need to be done every time the frame is rendered
    }
}

impl crate::visual::geometry::Transformable for BaseStimulus {
    fn set_transformation(&self, transformation: Transformation2D) {
        *self.transforms.lock_blocking() = transformation;
    }

    fn add_transformation(&self, transformation: Transformation2D) {
        let mut old_transformation = self.transforms.lock_blocking();
        let new_transformation = transformation * old_transformation.clone();
        *old_transformation = new_transformation;
    }
}

impl Stimulus for BaseStimulus {
    fn prepare(
        &mut self,
        window: &Window,
        window_state: &WindowState,
        gpu_state: &GPUState,
    ) {
        // if the stimulus is not visible we don't need to do anything
        if !self.visible.load_relaxed() {
            return;
        }

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
        // if the stimulus is not visible, return
        if !self.visible.load_relaxed() {
            return;
        }

        let pipeline = self.pipeline.lock_blocking();
        let n_vertices = self.n_vertices.load_relaxed();

        let tts_bind_group = self.tts_bind_group.lock_blocking();
        let bind_group = self.uniform_bind_group.lock_blocking();

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
            rpass.set_bind_group(0, &tts_bind_group, &[]);
            rpass.set_bind_group(1, &bind_group, &[]);

            rpass.draw(0..n_vertices as u32, 0..1);
        }
    }
}
