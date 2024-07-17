use lyon::{geom::Angle, path::Winding};

use super::geometry::{LineCap, LineJoin, Point2D, Primitive, TessellationOptions};

/// A vertex with position, color, and texture coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUVertex {
    pub position: Point2D,
    pub normal: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl GPUVertex {
    pub fn desc() -> &'static [wgpu::VertexAttribute] {
        &[
            wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                offset: 8,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                offset: 16,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 2,
            },
        ]
    }
}

pub struct GPUGeometryBuffer {
    pub vertices: Vec<GPUVertex>,
    pub indices: Vec<u32>,
    pub indices_offsets: Vec<u32>,
    pub indices_sizes: Vec<u32>,
}

impl lyon::tessellation::geometry_builder::GeometryBuilder for GPUGeometryBuffer {
    fn add_triangle(
        &mut self,
        a: lyon::tessellation::VertexId,
        b: lyon::tessellation::VertexId,
        c: lyon::tessellation::VertexId,
    ) {
        // Add the three vertices to the current geometry
        self.indices.push(a.0 as u32);
        self.indices.push(b.0 as u32);
        self.indices.push(c.0 as u32);
    }

    fn abort_geometry(&mut self) {
        // we cannot abort the geometry and need to panic
        panic!("Something went wrong while tessellating a geometry, cannot proceed.");
    }
}

impl lyon::tessellation::geometry_builder::FillGeometryBuilder for GPUGeometryBuffer {
    fn add_fill_vertex(
        &mut self,
        vertex: lyon::tessellation::FillVertex,
    ) -> Result<lyon::tessellation::VertexId, lyon::tessellation::GeometryBuilderError> {
        // Add the vertex to the vertex buffer
        self.vertices.push(GPUVertex {
            position: Point2D {
                x: vertex.position().x,
                y: vertex.position().y,
            },
            normal: [0.0, 0.0],
            tex_coords: [vertex.position().x, vertex.position().y],
        });

        Ok(lyon::tessellation::VertexId(
            (self.vertices.len() - 1) as u32,
        ))
    }
}

impl lyon::tessellation::geometry_builder::StrokeGeometryBuilder for GPUGeometryBuffer {
    fn add_stroke_vertex(
        &mut self,
        vertex: lyon::tessellation::StrokeVertex,
    ) -> Result<lyon::tessellation::VertexId, lyon::tessellation::GeometryBuilderError> {
        // Update the last vertex id
        //self.last_vertex_id = self.vertices.len() as u32;

        // Add the vertex to the vertex buffer
        self.vertices.push(GPUVertex {
            position: Point2D {
                x: vertex.position().x,
                y: vertex.position().y,
            },
            normal: [vertex.normal().x, vertex.normal().y],
            tex_coords: [vertex.position().x, vertex.position().y],
        });

        Ok(lyon::tessellation::VertexId(
            (self.vertices.len() - 1) as u32,
        ))
    }
}

impl GPUGeometryBuffer {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            indices_offsets: Vec::new(),
            indices_sizes: Vec::new(),
        }
    }

    pub fn tesselate(&mut self, primitive: &Primitive, options: &TessellationOptions) {
        match options {
            TessellationOptions::Fill => {
                self.tesselate_fill(primitive, options);
            }
            TessellationOptions::Stroke { .. } => {
                self.tesselate_stroke(primitive, options);
            }
        }
    }

    pub fn tesselate_stroke(&mut self, primitive: &Primitive, options: &TessellationOptions) {
        // create the tessellator
        let mut tessellator = lyon::tessellation::StrokeTessellator::new();

        // create the tessellation options
        let lyon_options = match options {
            TessellationOptions::Stroke {
                line_width,
                line_join,
                start_cap,
                end_cap,
                ..
            } => {
                let mut o = lyon::tessellation::StrokeOptions::default();
                o.line_width = *line_width;
                o.end_cap = match end_cap {
                    LineCap::Butt => lyon::tessellation::LineCap::Butt,
                    LineCap::Round => lyon::tessellation::LineCap::Round,
                    LineCap::Square => lyon::tessellation::LineCap::Square,
                };
                o.start_cap = match start_cap {
                    LineCap::Butt => lyon::tessellation::LineCap::Butt,
                    LineCap::Round => lyon::tessellation::LineCap::Round,
                    LineCap::Square => lyon::tessellation::LineCap::Square,
                };
                o.line_join = match line_join {
                    LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
                    LineJoin::Round => lyon::tessellation::LineJoin::Round,
                    LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
                    LineJoin::MiterClip => todo!(),
                };
                o
            }
            _ => panic!("Invalid tessellation options"),
        };

        // add the offset
        let indices_offset = self.indices.len() as u32;

        self.indices_offsets.push(indices_offset);

        match primitive {
            Primitive::Circle { center, radius } => {
                tessellator
                    .tessellate_circle(
                        lyon::math::Point::new(center.x, center.y),
                        *radius,
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            Primitive::Rectangle { a, b } => {
                tessellator
                    .tessellate_rectangle(
                        &lyon::math::Box2D::new(
                            lyon::math::Point::new(a.x, a.y),
                            lyon::math::Point::new(b.x, b.y),
                        ),
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            Primitive::Ellipse { center, radii } => {
                let rot_rad = Angle::degrees(0.0);
                tessellator
                    .tessellate_ellipse(
                        lyon::math::Point::new(center.x, center.y),
                        lyon::math::Vector::new(radii.x, radii.y),
                        rot_rad,
                        Winding::Positive,
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            Primitive::Line { a, b } => {
                tessellator
                    .tessellate_polygon(
                        lyon::path::Polygon {
                            points: &[
                                lyon::math::Point::new(a.x, a.y),
                                lyon::math::Point::new(b.x, b.y),
                            ],
                            closed: false,
                        },
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            _ => {
                todo!();
            }
        }

        // add the size
        self.indices_sizes
            .push((self.indices.len() - indices_offset as usize) as u32);
    }

    pub fn tesselate_fill(&mut self, primitive: &Primitive, _options: &TessellationOptions) {
        // create the tessellator
        let mut tessellator = lyon::tessellation::FillTessellator::new();

        // create the tessellation options
        let lyon_options = lyon::tessellation::FillOptions::tolerance(0.01);

        // add the offset
        let indices_offset = self.indices.len() as u32;

        self.indices_offsets.push(indices_offset);

        match primitive {
            Primitive::Circle { center, radius } => {
                tessellator
                    .tessellate_circle(
                        lyon::math::Point::new(center.x, center.y),
                        *radius,
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            Primitive::Rectangle { a, b } => {
                tessellator
                    .tessellate_rectangle(
                        &lyon::math::Box2D::new(
                            lyon::math::Point::new(a.x, a.y),
                            lyon::math::Point::new(b.x, b.y),
                        ),
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            Primitive::Ellipse { center, radii, .. } => {
                let rot_rad = Angle::degrees(0.0);
                tessellator
                    .tessellate_ellipse(
                        lyon::math::Point::new(center.x, center.y),
                        lyon::math::Vector::new(radii.x, radii.y),
                        rot_rad,
                        Winding::Positive,
                        &lyon_options,
                        self,
                    )
                    .unwrap();
            }
            _ => {
                panic!("Unsupported primitive type for tesselation");
            }
        }
        // update the texture coordinates by scaling them to the range [0, 1]
        let min_x = self
            .vertices
            .iter()
            .map(|v| v.position.x)
            .reduce(f32::min)
            .unwrap();
        let max_x = self
            .vertices
            .iter()
            .map(|v| v.position.x)
            .reduce(f32::max)
            .unwrap();

        let min_y = self
            .vertices
            .iter()
            .map(|v| v.position.y)
            .reduce(f32::min)
            .unwrap();
        let max_y = self
            .vertices
            .iter()
            .map(|v| v.position.y)
            .reduce(f32::max)
            .unwrap();

        for vertex in &mut self.vertices {
            vertex.tex_coords[0] = (vertex.position.x - min_x) / (max_x - min_x);
            vertex.tex_coords[1] = (vertex.position.y - min_y) / (max_y - min_y);
        }
        // add the size
        self.indices_sizes
            .push((self.indices.len() - indices_offset as usize) as u32);
    }
}
