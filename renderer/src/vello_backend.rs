use std::sync::Arc;

use crate::brushes::Extend;
use image::GenericImageView;
use vello::peniko::BlendMode;
use vello::RendererOptions;
use wgpu::util::DeviceExt;

use super::brushes::{Gradient, GradientKind, Image};
use super::scenes::SceneTrait;
use super::text::{Alignment, FormatedText, VerticalAlignment};
use crate::geoms::Geom;
use crate::prerenderd_scene::PrerenderedScene;
use crate::shapes::Shape;
use crate::styles::{CompositeMode, FillStyle, MixMode, StrokeOptions, Style};
use crate::{affine::Affine, scenes::Scene, Drawable};

use super::{
    brushes::{Brush, ColorStop},
    colors::RGBA,
    shapes::{Circle, Point, Rectangle, RoundedRectangle},
};


pub struct VelloBackend {
    /// The Vello scene.
    pub vello_scene: vello::Scene,
    /// The global transform.
    pub global_transform: Affine,
    /// array of
    pub gpu_images: Vec<(
        vello::peniko::Image,
        wgpu::ImageCopyTextureBase<Arc<wgpu::Texture>>,
    )>,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GammaParams {
    r: [f32; 8],
    g: [f32; 8],
    b: [f32; 8],
    correction: u32,
}

pub struct VelloRenderer {
    /// The vello renderer struct
    pub renderer: vello::Renderer,
    /// An optional wgpu render pipeline used for rendering the texture that vello produces
    pub render_pipeline: wgpu::RenderPipeline,
    /// The texture that vello will render to
    pub texture: wgpu::Texture,
    /// Uniform buffer for gamma correction
    pub gamma_buffer: wgpu::Buffer,
    /// The bind group
    pub bind_group: wgpu::BindGroup,
}

impl VelloRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let renderer = vello::Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(surface_format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: std::num::NonZeroUsize::new(1),
            },
        )
            .unwrap();


        // create a render pipeline
        let render_pipeline = Self::create_render_pipelie(width, height, device, surface_format);
        let texture = Self::create_texture(device, width, height);
        let gamma_buffer = Self::create_uniform_buffer(device);
        let bind_group = Self::create_bind_group(device, &texture);

        Self {
            renderer,
            render_pipeline,
            texture,
            gamma_buffer,
            bind_group,
        }
    }

    /// Re-size the texture
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = Self::create_texture(device, width, height);
        self.bind_group = Self::create_bind_group(device, &self.texture);
    }

    /// Render the scene to a WGPU surface.
    pub fn render_to_surface(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::SurfaceTexture,
        scene: &Scene<VelloBackend>,
    ) {
        let vello_scene = &scene.backend.vello_scene;
        let render_params = vello::RenderParams {
            base_color: scene.background_color.into(),
            width: surface.texture.width(),
            height: surface.texture.height(),
            antialiasing_method: vello::AaConfig::Msaa16,
        };
        // (interim) replace the images with GPU textures.
        for (image, wgpu_texture) in &scene.backend.gpu_images {
            self.renderer
                .override_image(image, Some(wgpu_texture.clone()));
        }
        self.renderer
            .render_to_surface(device, queue, vello_scene, surface, &render_params);
    }

    /// Render the scene to a WGPU texture.
    pub fn render_to_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::TextureView,
        width: u32,
        height: u32,
        scene: &Scene<VelloBackend>,
    ) {
        // print the texture format

        let vello_scene = &scene.backend.vello_scene;
        let render_params = vello::RenderParams {
            base_color: scene.background_color.into(),
            width: width,
            height: height,
            antialiasing_method: vello::AaConfig::Msaa16,
        };

        // (interim) replace the images with GPU textures.
        for (image, wgpu_texture) in &scene.backend.gpu_images {
            self.renderer
                .override_image(image, Some(wgpu_texture.clone()));
        }
        self.renderer
            .render_to_texture(device, queue, vello_scene, texture, &render_params).expect("Failed to render to texture");
    }

    /// Render the scene to a WGPU surface but sets up its own render pass.
    pub fn render_to_surface2(&mut self,
                              device: &wgpu::Device,
                              queue: &wgpu::Queue,
                              surface: &wgpu::SurfaceTexture,
                              scene: &Scene<VelloBackend>,
    ) {
        // create texture view
        let texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // render the scene
        self.render_to_texture(device, queue, &texture_view, surface.texture.width(), surface.texture.height(), scene);


        // create a new render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });


        let surface_texture_view = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());


        {
            // bind the render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // bind the render pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            // bind the bind group
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            // draw the quad
            render_pass.draw(0..6, 0..1);
        }

        // submit the render pass
        queue.submit(Some(encoder.finish()));
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        })
    }

    fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gamma Buffer"),
            size: std::mem::size_of::<GammaParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_bind_group(device: &wgpu::Device, texture: &wgpu::Texture) -> wgpu::BindGroup {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Gamma Buffer"),
                            contents: bytemuck::cast_slice(&[GammaParams {
                                correction: 3, // 0: none, 1: psychopy, 2: polylog4, 3: polylog5, 4: polylog6
                                r: [0.9972361456765942, 0.5718201120693766, 0.1494526003308258, 0.021348959590415988, 0.0016066519145011171, 4.956890077371443e-05, 0.0, 0.0],
                                g: [1.0058002029776596, 0.5695706025327177, 0.14551632725612368, 0.020115266744271217, 0.0014548822571441762, 4.3086307473990124e-05, 0.0, 0.0],
                                b: [1.0116733520722856, 0.5329488652553003, 0.11728724922990535, 0.012259928984426039, 0.000528402626505164, 4.086604661837748e-06, 0.0, 0.0],
                            }]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        }),
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }

    fn create_render_pipelie(width: u32, height: u32, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/shaders/render.wgsl").into()),
        });

        // create a bind group layout for texture and sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });


        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: &"vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: &"fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
        });


        render_pipeline
    }
}

impl VelloBackend {
    /// Create a new Vello backend.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            vello_scene: vello::Scene::new(),
            global_transform: Affine::translate(width as f64 / 2.0, height as f64 / 2.0),
            gpu_images: Vec::new(),

        }
    }
}

impl Scene<VelloBackend> {
    /// Create a new scene.
    pub fn new(background_color: RGBA, width: u32, height: u32) -> Self {
        Self {
            background_color,
            width,
            height,
            backend: VelloBackend::new(width, height),
        }
    }

    /// draw a renderable object.
    pub fn draw(&mut self, mut object: impl Drawable<VelloBackend>) {
        // Draw the object.
        object.draw(self);
    }
}

// Textures
impl Image {
    /// Create a new texture from an image::DynamicImage.
    pub fn new(image: &image::DynamicImage) -> Self {
        // create a peniko image
        let data = Arc::new(image.clone().into_rgba8().into_vec());

        return Self {
            gpu_texture: None,
            data,
            width: image.width(),
            height: image.height(),
        };
    }

    /// Move the texture to the GPU.
    pub fn to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let data = &self.data;
        // create a new wgpu texture
        let wgpu_tetxure = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        // write the image to the texture
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_tetxure,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.gpu_texture = Some(Arc::new(wgpu_tetxure));
    }
}

impl<S: IntoVelloShape + Shape> Drawable<VelloBackend> for Geom<S> {
    fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
        let transform = (scene.backend.global_transform * self.transform).into();

        let brush_transform = self.brush_transform.map(|t| t.into());

        // convert the brush
        let new_brush = &self.brush.as_brush_or_brushref();

        // if brush is an image
        if let Brush::Image { image, .. } = &self.brush {
            if let Some(gpu_texture) = &image.gpu_texture {
                scene.backend.gpu_images.push((
                    new_brush.clone().try_into().unwrap(),
                    wgpu::ImageCopyTextureBase {
                        texture: gpu_texture.clone(),
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                ));
            }
        }

        let shape = &self.shape.clone().into_vello_shape();
        // match the style (stroke or fill)

        match self.style.clone() {
            Style::Fill(style) => {
                // fill the shape
                scene.backend.vello_scene.fill(
                    style.into(),
                    transform,
                    new_brush,
                    brush_transform,
                    &shape,
                );
            }
            Style::Stroke(style) => {
                scene.backend.vello_scene.stroke(
                    &style.into(),
                    transform,
                    new_brush,
                    brush_transform,
                    &shape,
                );
            }
        }
    }
}

impl<ClipShape: IntoVelloShape + Shape> SceneTrait<VelloBackend, ClipShape>
for Scene<VelloBackend>
{
    fn scene_mut(&mut self) -> &mut Scene<VelloBackend> {
        self
    }

    fn scene(&self) -> &Scene<VelloBackend> {
        self
    }

    fn start_layer(
        &mut self,
        mix_mode: MixMode,
        composite_mode: CompositeMode,
        clip: ClipShape,
        clip_transform: Affine,
        layer_transform: Option<Affine>,
        alpha: f32,
    ) {
        // error if a layer transform is provided
        if layer_transform.is_some() {
            todo!();
        }
        let clip_shape = clip.into_vello_shape();
        let global_transform = self.backend.global_transform;
        let clip_transform = (global_transform * clip_transform).into();

        self.backend.vello_scene.push_layer(
            BlendMode::new(mix_mode.into(), composite_mode.into()),
            alpha,
            clip_transform,
            &clip_shape,
        );
    }

    fn end_layer(&mut self) {
        self.backend.vello_scene.pop_layer();
    }
}

// allow converting different types into the vello types

// Point2D
impl From<Point> for vello::kurbo::Point {
    fn from(point: Point) -> Self {
        vello::kurbo::Point::new(point.x, point.y)
    }
}

// Affine
impl From<Affine> for vello::kurbo::Affine {
    fn from(affine: Affine) -> Self {
        vello::kurbo::Affine::new(affine.0)
    }
}

// FillStyle
impl From<FillStyle> for vello::peniko::Fill {
    fn from(style: FillStyle) -> Self {
        match style {
            FillStyle::NonZero => vello::peniko::Fill::NonZero,

            FillStyle::EvenOdd => vello::peniko::Fill::EvenOdd,
        }
    }
}

// StrokeStyle
impl From<StrokeOptions> for vello::kurbo::Stroke {
    fn from(style: StrokeOptions) -> Self {
        vello::kurbo::Stroke {
            width: style.width,
            ..Default::default()
        }
    }
}

// BrushRef (this needs to be refactored)
impl<'a> Brush {
    fn as_brush_or_brushref(&'a self) -> VelloBrushOrBrushRef<'a> {
        match self {
            Brush::Image { image, fit_mode, edge_mode, x, y } => {
                // note that offsets and fit mode are already applied when the geom is created and part
                // of the brush transform

                // create peniko::Image
                let blob = vello::peniko::Blob::new(image.data.clone());
                let image = vello::peniko::Image::new(blob, vello::peniko::Format::Rgba8, image.width, image.height);
                let image = image.with_extend(edge_mode.into());

                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Image(image))
            }
            Brush::Solid(rgba) => {
                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Solid(rgba.clone().into()))
            }
            Brush::Gradient(gradient) => {
                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Gradient(gradient.clone().into()))
            }
        }
    }
}

/// A brush or a brush reference
pub enum VelloBrushOrBrushRef<'a> {
    /// A brush brush
    Brush(vello::peniko::Brush),
    /// A brush reference
    BrushRef(vello::peniko::BrushRef<'a>),
}

// allow VelloBrushOrBrushRef to BrushRef
impl<'a> From<&'a VelloBrushOrBrushRef<'a>> for vello::peniko::BrushRef<'a> {
    fn from(brush: &'a VelloBrushOrBrushRef<'a>) -> Self {
        match brush {
            VelloBrushOrBrushRef::BrushRef(brush_ref) => brush_ref.clone(),
            VelloBrushOrBrushRef::Brush(brush) => brush.into(),
        }
    }
}

// allow to get peniko::Image from VelloBrushOrBrushRef
impl<'a> TryFrom<&'a VelloBrushOrBrushRef<'a>> for vello::peniko::Image {
    type Error = &'static str;

    fn try_from(brush: &'a VelloBrushOrBrushRef<'a>) -> Result<Self, Self::Error> {
        match brush {
            VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Image(image)) => Ok(image.clone()),
            _ => Err("Not an image brush"),
        }
    }
}

// implement vello Shape trait for different shapes
trait IntoVelloShape {
    type VelloShape: vello::kurbo::Shape;
    fn into_vello_shape(self) -> Self::VelloShape;
}

// rectangle
impl IntoVelloShape for Rectangle {
    type VelloShape = vello::kurbo::Rect;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::Rect::new(self.a.x, self.a.y, self.b.x, self.b.y)
    }
}

// rounded rectangle
impl IntoVelloShape for RoundedRectangle {
    type VelloShape = vello::kurbo::RoundedRect;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::RoundedRect::new(self.a.x, self.a.y, self.b.x, self.b.y, self.radius)
    }
}

// circle
impl IntoVelloShape for Circle {
    type VelloShape = vello::kurbo::Circle;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::Circle::new(self.center, self.radius)
    }
}

// Colors
impl From<RGBA> for vello::peniko::Color {
    fn from(color: RGBA) -> Self {
        vello::peniko::Color::rgba(
            color.r as f64,
            color.g as f64,
            color.b as f64,
            color.a as f64,
        )
    }
}

// MixMode
impl From<MixMode> for vello::peniko::Mix {
    fn from(mode: MixMode) -> Self {
        match mode {
            MixMode::Normal => vello::peniko::Mix::Normal,
            MixMode::Clip => vello::peniko::Mix::Clip,
            MixMode::Multiply => vello::peniko::Mix::Multiply,
        }
    }
}

// CompositeMode
impl From<CompositeMode> for vello::peniko::Compose {
    fn from(mode: CompositeMode) -> Self {
        match mode {
            CompositeMode::SourceIn => vello::peniko::Compose::SrcIn,
            CompositeMode::SourceOut => vello::peniko::Compose::SrcOut,
            CompositeMode::SourceOver => vello::peniko::Compose::SrcOver,
            CompositeMode::DestinationOver => vello::peniko::Compose::DestOver,
            CompositeMode::DestinationIn => vello::peniko::Compose::DestIn,
            CompositeMode::DestinationOut => vello::peniko::Compose::DestOut,
            CompositeMode::DestinationAtop => vello::peniko::Compose::DestAtop,
            CompositeMode::Xor => vello::peniko::Compose::Xor,
            CompositeMode::SourceAtop => vello::peniko::Compose::SrcAtop,
            CompositeMode::Lighter => vello::peniko::Compose::PlusLighter,
            CompositeMode::Copy => vello::peniko::Compose::Copy,
        }
    }
}

// ColorStop
impl From<ColorStop> for vello::peniko::ColorStop {
    fn from(color_stop: ColorStop) -> Self {
        vello::peniko::ColorStop {
            offset: color_stop.offset,
            color: color_stop.color.into(),
        }
    }
}

// Extend
impl From<Extend> for vello::peniko::Extend {
    fn from(extend: Extend) -> Self {
        match extend {
            Extend::Pad => vello::peniko::Extend::Pad,
            Extend::Repeat => vello::peniko::Extend::Repeat,
            Extend::Reflect => vello::peniko::Extend::Reflect,
        }
    }
}

impl From<&Extend> for vello::peniko::Extend {
    fn from(extend: &Extend) -> Self {
        match extend {
            Extend::Pad => vello::peniko::Extend::Pad,
            Extend::Repeat => vello::peniko::Extend::Repeat,
            Extend::Reflect => vello::peniko::Extend::Reflect,
        }
    }
}

// GradientKind
impl From<GradientKind> for vello::peniko::GradientKind {
    fn from(kind: GradientKind) -> Self {
        match kind {
            GradientKind::Linear { start, end } => vello::peniko::GradientKind::Linear {
                start: start.into(),
                end: end.into(),
            },
            GradientKind::Radial {
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => vello::peniko::GradientKind::Radial {
                start_center: start_center.into(),
                start_radius,
                end_center: end_center.into(),
                end_radius,
            },
            GradientKind::Sweep {
                center,
                start_angle,
                end_angle,
            } => vello::peniko::GradientKind::Sweep {
                center: center.into(),
                start_angle,
                end_angle,
            },
        }
    }
}

// Gradient
impl From<Gradient> for vello::peniko::Gradient {
    fn from(gradient: Gradient) -> Self {
        vello::peniko::Gradient {
            kind: gradient.kind.into(),
            stops: gradient.stops.into_iter().map(|stop| stop.into()).collect(),
            extend: gradient.extend.into(),
        }
    }
}

// Text
#[derive(Debug, Clone)]
pub struct VelloFont(vello::peniko::Font);

impl VelloFont {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let blob = vello::peniko::Blob::new(Arc::new(bytes.to_vec()));
        let font = vello::peniko::Font::new(blob, 0);

        Self(font)
    }
}

impl Drawable<VelloBackend> for FormatedText<VelloFont> {
    fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
        let transform: vello::kurbo::Affine =
            (self.transform * scene.backend.global_transform).into();

        let font = &self.font.0;
        let font_size = vello::skrifa::instance::Size::new(self.size);
        let text = &self.text;

        let font_ref = vello_font_to_font_ref(font).expect("Failed to load font");
        let axes = vello::skrifa::MetadataProvider::axes(&font_ref);
        let variations = [("wght", 100.0), ("wdth", 500.0)];
        let var_loc = axes.location(variations.iter().copied());

        let charmap = vello::skrifa::MetadataProvider::charmap(&font_ref);
        let metrics = vello::skrifa::MetadataProvider::metrics(&font_ref, font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics =
            vello::skrifa::MetadataProvider::glyph_metrics(&font_ref, font_size, &var_loc);

        let mut pen_x = (self.x * 2.0) as f32;
        let mut pen_y = (self.y * 2.0) as f32;

        let brush_color: vello::peniko::Color = self.color.into();

        let glyphs = text
            .chars()
            .filter_map(|ch| {
                if ch == '\n' {
                    pen_y += line_height;
                    pen_x = 0.0;
                    return None;
                }
                let gid = charmap.map(ch).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;
                Some(vello::Glyph {
                    id: gid.to_u32(),
                    x,
                    y: pen_y,
                })
            })
            .collect::<Vec<_>>();

        let text_width = pen_x as f64;
        let text_height = pen_y as f64 + line_height as f64;

        let transform_x = match self.alignment {
            Alignment::Left => 0.0,
            Alignment::Center => -text_width / 2.0,
            Alignment::Right => -text_width,
        };

        let transform_y = match self.vertical_alignment {
            VerticalAlignment::Top => 0.0,
            VerticalAlignment::Middle => text_height / 2.0,
            VerticalAlignment::Bottom => text_height,
        };

        let transform = transform.pre_translate(vello::kurbo::Vec2::new(transform_x, transform_y));

        scene
            .backend
            .vello_scene
            .draw_glyphs(font)
            .font_size(self.size)
            .transform(transform)
            .glyph_transform(self.glyph_transform.map(|t| t.into()))
            .normalized_coords(var_loc.coords())
            .brush(brush_color)
            .hint(false)
            .draw(vello::peniko::Fill::NonZero, glyphs.into_iter());
    }
}

fn vello_font_to_font_ref(font: &vello::peniko::Font) -> Option<vello::skrifa::FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

impl Drawable<VelloBackend> for &PrerenderedScene {
    fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
        let global_transform = scene.backend.global_transform;
        let transform = self.transform * global_transform;

        scene.backend.vello_scene.append(&mut &self.scene, Some(transform.into()));
    }
}