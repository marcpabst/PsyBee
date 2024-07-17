use std::collections::HashMap;
use std::num::NonZeroU64;

use geometry::Geom;

use geometry::Transformation;
use helpers::Cache;
use material::TextureFilter;
use material::TextureRepeat;
use material::{Material, MaterialType};

use texture::Texture;
use texture::TextureFormat;
use uniform_structs::ScreenUniforms;
use vertex::GPUGeometryBuffer;
use vertex::GPUVertex;
use wgpu;

pub mod geometry;
pub mod helpers;
pub mod material;
pub mod texture;
pub mod uniform_structs;
pub mod vertex;

const VERRTEX_BUFFER_SIZE_MB: u32 = 20;
const INDEX_BUFFER_SIZE_MB: u32 = 20;
const UNIFORM_BUFFER_SIZE_MB: u32 = 20;

pub type CachedTesselation = (Vec<GPUVertex>, Vec<u32>);
pub type CachedTexture = (wgpu::Buffer, wgpu::Texture, wgpu::TextureView);

pub struct Renderer {
    /// A HashMap mapping material types to material instances.
    materials: HashMap<MaterialType, MaterialInstance>,
    /// The global vertex buffer.
    vertex_buffer: wgpu::Buffer,
    /// The global index buffer.
    index_buffer: wgpu::Buffer,
    /// The global uniform buffer.
    uniform_buffer: wgpu::Buffer,
    /// The global bind group layout.
    bind_group_layout: wgpu::BindGroupLayout,
    /// The global bind group.
    bind_group: wgpu::BindGroup,
    /// Global tesselation cache.
    #[allow(dead_code)]
    tesselation_cache: Cache<CachedTesselation>,
    /// Global texture cache.
    texture_cache: Cache<CachedTexture>,
}

pub struct RenderData {
    pub index_buffer_offsets: Vec<u32>,
    pub index_buffer_sizes: Vec<u32>,
    pub uniform_buffer_offsets: Vec<u32>,
    pub texture_bind_groups: Vec<Option<wgpu::BindGroup>>,
}

/// A renderable object.
pub enum Renderable {
    /// A primitive which is directly handled by the renderer.
    Primitive(Geom),
    // /// A GUI element which is handled by gui renderer.
    // GuiElement(GuiElement),
    // /// A text element which is handled by the text renderer.
    // TextElement(TextElement),
}

pub struct MaterialInstance {
    pub vertex_shader: wgpu::ShaderModule,
    pub fragment_shader: wgpu::ShaderModule,
    pub pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub texture_sampler: Option<wgpu::Sampler>,
}

impl Renderer {
    /// Creates a new primitive renderer.
    pub fn new(device: &wgpu::Device) -> Self {
        // Create the global vertex buffer that will store all the vertices for all the primitives.
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (VERRTEX_BUFFER_SIZE_MB * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the global index buffer that will store all the indices for all the primitives.
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (INDEX_BUFFER_SIZE_MB * 1024) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the global uniform buffer that will store all the uniform data for all the primitives.
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: (UNIFORM_BUFFER_SIZE_MB * 1024) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the the global bind group for the uniform buffer.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            // The binding for the uniform buffer.
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // buffer 1 contains uniform data
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: Some(NonZeroU64::new(256).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: Some(NonZeroU64::new(256).unwrap()),
                    }),
                },
            ],
            label: Some("Global uniform bind Group"),
        });

        // Create the material cache.
        let materials = HashMap::<MaterialType, MaterialInstance>::new();

        Self {
            materials,
            vertex_buffer,
            uniform_buffer,
            index_buffer,
            bind_group_layout,
            bind_group,
            tesselation_cache: Cache::new(),
            texture_cache: Cache::new(),
        }
    }

    /// Adds a texture to the cache. If the texture is already in the cache, this is a no-op.
    /// If the texture is in the cache, but the texture has changed, the texture is updated using the same buffer.
    pub fn add_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &Texture) {
        if self.texture_cache.get_sweep(texture).is_some() {
            // this will also remove the texture from the cache
            // if the fingerprint has changed
            return;
        }

        let texture_format = match texture.format() {
            TextureFormat::Srgba8U => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8U => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba32F => wgpu::TextureFormat::Rgba32Float,
        };

        // create the texture
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: texture.width(),
                height: texture.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[texture_format],
        });

        // create the texture view
        let texture_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // create the texture buffer
        let texture_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Buffer"),
            size: (texture.width() * texture.height() * 4) as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // upload the texture data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture.bytes_per_row()),
                rows_per_image: Some(texture.height()),
            },
            wgpu::Extent3d {
                width: texture.width(),
                height: texture.height(),
                depth_or_array_layers: 1,
            },
        );

        // add the texture to the cache
        self.texture_cache
            .insert(texture, (texture_buffer, gpu_texture, texture_view));
    }

    /// Prepares the bind group given a texture and a material instance.
    pub fn get_texture_bind_group(
        &self,
        device: &wgpu::Device,
        texture: &Texture,
        material_instance: &MaterialInstance,
    ) -> wgpu::BindGroup {
        // If the texture is not in the cache, we error out (this should be impossible).
        let (_texture_buffer, _texture, texture_view) = self
            .texture_cache
            .get(texture)
            .expect("Texture not in cache. This should not happen.");

        // get the texture sampler from the material instance
        let texture_sampler = material_instance
            .texture_sampler
            .as_ref()
            .expect("Material instance does not have a texture sampler. This should not happen.");

        // get the texture bind group layout from the material instance
        let texture_bind_group_layout = material_instance
            .texture_bind_group_layout
            .as_ref()
            .expect(
            "Material instance does not have a texture bind group layout. This should not happen.",
        );

        // create the texture bind group
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture_sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        texture_bind_group
    }

    /// Add a material type to the renderer. If the material type is already in the renderer, this is a no-op.
    pub fn add_material(&mut self, device: &wgpu::Device, material: Material) {
        // Check if the material type is already in the renderer.
        if self.materials.contains_key(&material.material_type()) {
            return;
        }

        // create the pipeline layout
        let pipeline_layout;
        let mut texture_sampler = None;
        let mut texture_bind_group_layout = None;

        if material.has_texture() {
            let filter = material
                .texture_filter()
                .expect("Material does not have texture filter. This should not happen.");

            let gpu_filter = match filter {
                TextureFilter::Nearest => wgpu::FilterMode::Nearest,
                TextureFilter::Linear => wgpu::FilterMode::Linear,
            };

            let sampler_binding_type = match gpu_filter {
                wgpu::FilterMode::Nearest => wgpu::SamplerBindingType::NonFiltering,
                wgpu::FilterMode::Linear => wgpu::SamplerBindingType::Filtering,
            };

            let _texture_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float {
                                    filterable: (gpu_filter == wgpu::FilterMode::Linear),
                                },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(sampler_binding_type),
                            count: None,
                        },
                    ],
                });

            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&self.bind_group_layout, &_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

            // get address modes from material
            let repeat_modes = material
                .texture_repeat_modes()
                .expect("Material does not have texture repeat modes. This should not happen.");

            let address_mode_u = match repeat_modes.0 {
                TextureRepeat::Repeat => wgpu::AddressMode::Repeat,
                TextureRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
                TextureRepeat::Mirror => wgpu::AddressMode::MirrorRepeat,
                TextureRepeat::None => wgpu::AddressMode::ClampToBorder,
            };

            let address_mode_v = match repeat_modes.1 {
                TextureRepeat::Repeat => wgpu::AddressMode::Repeat,
                TextureRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
                TextureRepeat::Mirror => wgpu::AddressMode::MirrorRepeat,
                TextureRepeat::None => wgpu::AddressMode::ClampToBorder,
            };

            texture_sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture Sampler"),
                address_mode_u: address_mode_u,
                address_mode_v: address_mode_v,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: gpu_filter,
                min_filter: gpu_filter,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
            }));

            texture_bind_group_layout = Some(_texture_bind_group_layout);
        } else {
            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&self.bind_group_layout],
                push_constant_ranges: &[],
            });
        };

        // Create the shader modules for the material.
        let vertex_shader = material.vertex_shader_module(device);
        let fragment_shader = material.fragment_shader_module(device);

        // create the pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GPUVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: GPUVertex::desc(),
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.materials.insert(
            material.material_type(),
            MaterialInstance {
                vertex_shader,
                fragment_shader,
                pipeline,
                texture_bind_group_layout,
                texture_sampler,
            },
        );
    }

    /// Prepare the renderer for rendering.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_desc: &wgpu::SurfaceConfiguration,
        geoms: &[Geom],
    ) -> RenderData {
        let mut draw_buffer_collector = GPUGeometryBuffer::new();

        let offset_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;

        let mut uniform_buffer_offsets: Vec<u32> = vec![2 * offset_alignment as u32];
        let mut texture_bind_groups: Vec<Option<wgpu::BindGroup>> = vec![];

        // calculate total size of the uniform buffer
        let uniform_buffer_size = geoms.len() * offset_alignment + 2 * offset_alignment;

        for geom in geoms {
            // add material
            self.add_material(&device, geom.material.clone());
        }

        {
            let mut staging_buffer = queue
                .write_buffer_with(
                    &self.uniform_buffer,
                    0,
                    NonZeroU64::new(uniform_buffer_size as u64).unwrap(),
                )
                .expect("Failed to write buffer");

            // write screen uniforms
            let screen_uniforms = ScreenUniforms {
                screen_width: surface_desc.width,
                screen_height: surface_desc.height,
            };

            staging_buffer[0..std::mem::size_of::<ScreenUniforms>()]
                .copy_from_slice(bytemuck::bytes_of(&screen_uniforms));

            // prepare the draw buffer
            for geom in geoms {
                draw_buffer_collector.tesselate(&geom.primitive, &geom.options);

                // as primitive uniforms
                let mut primitive_uniforms = Vec::<u8>::with_capacity(80);
                primitive_uniforms.extend(bytemuck::bytes_of(
                    &geom.transform.unwrap_or(Transformation::identity()),
                ));

                // add the bbox (min, max)
                primitive_uniforms.extend(bytemuck::bytes_of(&geom.primitive.bbox()));

                let primitive_uniforms_len = primitive_uniforms.len() as usize;

                // add material uniforms
                let material_uniforms = geom.material.uniform_bytes();

                let current_uniform_offset = uniform_buffer_offsets.last().unwrap().clone();

                // lenght must be a multiple of the alignment
                let current_uniform_length =
                    (material_uniforms.len() + primitive_uniforms_len + offset_alignment - 1)
                        & !(offset_alignment - 1);

                // copy the uniforms into the buffer at the correct offset
                staging_buffer[(current_uniform_offset as usize)
                    ..(current_uniform_offset as usize + primitive_uniforms_len as usize)]
                    .copy_from_slice(bytemuck::cast_slice(primitive_uniforms.as_slice()));

                staging_buffer[(current_uniform_offset as usize + primitive_uniforms_len)
                    ..(current_uniform_offset as usize
                        + material_uniforms.len()
                        + primitive_uniforms_len as usize)]
                    .copy_from_slice(material_uniforms.as_slice());

                // add the offset to the list
                uniform_buffer_offsets.push(current_uniform_offset + current_uniform_length as u32);
            }
        }

        // handle textures

        for geom in geoms {
            // if the material has a texture, we need to add the texture to the renderer
            if let Some(texture) = geom.material.texture() {
                self.add_texture(&device, queue, texture);

                let material = self
                    .materials
                    .get(&geom.material.material_type())
                    .expect("Material not found");

                texture_bind_groups.push(Some(
                    self.get_texture_bind_group(&device, &texture, &material),
                ));
            } else {
                texture_bind_groups.push(None);
            }
        }

        // Write the vertex buffer data.
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&draw_buffer_collector.vertices),
        );

        // Write the index buffer data.
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&draw_buffer_collector.indices),
        );

        RenderData {
            index_buffer_offsets: draw_buffer_collector.indices_offsets,
            index_buffer_sizes: draw_buffer_collector.indices_sizes,
            uniform_buffer_offsets: uniform_buffer_offsets,
            texture_bind_groups: texture_bind_groups,
        }
    }

    /// Render.
    pub fn render<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        _device: &wgpu::Device,
        primitives: &'rpass [Geom],
        rdata: &'rpass RenderData,
    ) {
        // Set the global vertex and index buffers.
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        let mut last_material = None;

        for (i, primitive) in primitives.iter().enumerate() {
            let material = self
                .materials
                .get(&primitive.material.material_type())
                .expect("Material not found");

            if last_material != Some(primitive.material.material_type()) {
                // Set the pipeline.
                rpass.set_pipeline(&material.pipeline);
                last_material = Some(primitive.material.material_type());
            }

            let uniform_offset = rdata.uniform_buffer_offsets[i];
            rpass.set_bind_group(0, &self.bind_group, &[0, uniform_offset as u32]);

            // if the material has a texture, we need to bind the extra bind group
            if let Some(..) = primitive.material.texture() {
                let texture_bind_group = rdata.texture_bind_groups[i]
                    .as_ref()
                    .expect("Texture bind group not found");
                rpass.set_bind_group(1, &texture_bind_group, &[]);
            }

            // Draw
            let index_buffer_offset = rdata.index_buffer_offsets[i];
            let index_buffer_size = rdata.index_buffer_sizes[i];

            rpass.draw_indexed(
                index_buffer_offset..(index_buffer_offset + index_buffer_size as u32),
                0,
                0..1,
            );
        }
    }
}
