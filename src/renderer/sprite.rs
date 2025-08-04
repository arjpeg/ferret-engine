use std::collections::HashMap;

use glam::{Mat4, vec2, vec4};
use wgpu::{util::*, wgt::BufferDescriptor, *};

use crate::renderer::{
    pipelines::Pipelines,
    vertex::{SpriteInstance, SpriteVertex},
};
use crate::transform::Transform;

/// A 2D mesh used for rendering.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Mesh2D(pub Shape2D);

/// A 2D primitive that represenets some basic geometry.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Shape2D {
    /// A rectangle centered at the origin with corners at (-1, -1), (1, 1).
    Square,
}

/// A material used for rendering a 2D sprite
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Material2D {
    /// The entire sprite is shaded a flat color.
    FlatColor {
        /// The red component in the range [0, 1].
        r: f32,
        /// The green component in the range [0, 1].
        g: f32,
        /// The blue component in the range [0, 1].
        b: f32,
    },
}

/// The main 2D sprite renderer, responsible for efficiently batching 2D geometry.
pub(crate) struct SpriteRenderer {
    /// The geometry mesh for squares.
    square_mesh: GeometryMesh,

    /// The buffer holding all information for sprites currently being rendered.
    instance_buffer: Buffer,

    /// The bind group holding the camera's transformation matrix.
    camera_bind_group: BindGroup,
    /// The uniform buffer holding the camera's transformation matrix.
    camera_buffer: Buffer,
}

/// The base geometry for a 2D primitive shape.
#[derive(Debug)]
struct GeometryMesh {
    /// The buffer holding all vertices.
    vertex_buffer: Buffer,
    /// The buffer holding all indices.
    index_buffer: Buffer,
    /// The number of indices held in the mesh to be rendered.
    index_count: u32,
}

impl SpriteRenderer {
    /// The maximum count of sprites per render batch.
    pub const MAX_SPRITES_PER_BATCH: u64 = 100;

    /// Creates a new [`SpriteRenderer`].
    pub fn new(device: &Device, pipelines: &Pipelines) -> Self {
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("SpriteRenderer::transform_buffer"),
            size: Self::MAX_SPRITES_PER_BATCH * size_of::<Transform>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let square_mesh = GeometryMesh::new(
            device,
            "SpriteRenderer::square_mesh",
            &[
                SpriteVertex {
                    position: vec2(-1.0, 1.0),
                    color: [0.5, 0.4, 0.6],
                },
                SpriteVertex {
                    position: vec2(-1.0, -1.0),
                    color: [0.5, 0.4, 0.6],
                },
                SpriteVertex {
                    position: vec2(1.0, -1.0),
                    color: [0.5, 0.4, 0.6],
                },
                SpriteVertex {
                    position: vec2(1.0, 1.0),
                    color: [0.5, 0.4, 0.6],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        );

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("SpriteRenderer::camera_buffer"),
            size: size_of::<Mat4>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("SpriteRenderer::camera_bind_group"),
            layout: &pipelines.camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            square_mesh,
            instance_buffer,
            camera_bind_group,
            camera_buffer,
        }
    }

    /// Renders all the provided sprites to the current render pass.
    pub fn render(
        &mut self,
        rpass: &mut RenderPass<'_>,
        queue: &Queue,
        pipelines: &Pipelines,
        camera_transformation: Mat4,
        sprites: Vec<(Mesh2D, Material2D, Transform)>,
    ) {
        rpass.set_pipeline(&pipelines.sprite_render_pipeline);

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera_transformation),
        );

        let mut sprites_by_shape = HashMap::<Shape2D, Vec<SpriteInstance>>::new();

        for (Mesh2D(shape), material, transform) in sprites {
            sprites_by_shape
                .entry(shape)
                .or_default()
                .push(SpriteInstance {
                    transform: transform.as_model_matrix(),
                    color: match material {
                        Material2D::FlatColor { r, g, b } => vec4(r, g, b, 1.0),
                    },
                });
        }

        // shape: (first instance, length)
        let mut sprite_instance_ranges = HashMap::<Shape2D, (u32, u32)>::new();

        let instances =
            sprites_by_shape
                .into_iter()
                .fold(Vec::new(), |mut instances, (shape, sprites)| {
                    sprite_instance_ranges
                        .insert(shape, (instances.len() as u32, sprites.len() as u32));

                    instances.extend(sprites);
                    instances
                });

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        rpass.set_bind_group(0, &self.camera_bind_group, &[]);
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        for (shape, (first_instance, count)) in sprite_instance_ranges {
            let mesh = match shape {
                Shape2D::Square => &self.square_mesh,
            };

            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);

            rpass.draw_indexed(0..mesh.index_count, 0, first_instance..count as _);
        }
    }
}

impl GeometryMesh {
    /// Creates and uploads a mesh based on the given vertices and indices.
    fn new(
        device: &Device,
        label_prefix: &str,
        vertices: &[SpriteVertex],
        indices: &[u16],
    ) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{label_prefix}::vertex_buffer")),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{label_prefix}::index_buffer")),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            }),
            index_count: indices.len() as _,
        }
    }
}
