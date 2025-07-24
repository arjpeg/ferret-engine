use std::collections::HashMap;

use glam::vec2;
use wgpu::{util::*, *};

use crate::{
    component::{Mesh2D, Shape2D, Transform},
    renderer::{pipelines::Pipelines, vertex::SpriteVertex},
};

/// The main 2D sprite renderer, responsible for efficiently batching 2D geometry.
pub struct SpriteRenderer {
    /// The geometry mesh for rectangles.
    rectangle_mesh: GeometryMesh,

    /// The buffer holding all transforms for sprites currently being rendered.
    transform_buffer: Buffer,
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
    pub fn new(device: &Device) -> Self {
        let transform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("SpriteRenderer::transform_buffer"),
            size: Self::MAX_SPRITES_PER_BATCH * size_of::<Transform>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rectangle_mesh = GeometryMesh::new(
            device,
            "SpriteRenderer::rectangle_mesh",
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

        Self {
            rectangle_mesh,
            transform_buffer,
        }
    }

    /// Renders all the provided sprites to the current render pass.
    pub fn render(
        &mut self,
        pipelines: &Pipelines,
        rpass: &mut RenderPass<'_>,
        queue: &Queue,
        sprites: Vec<(Mesh2D, Transform)>,
    ) {
        rpass.set_pipeline(&pipelines.sprite_render_pipeline);

        let mut sprites_by_shape = HashMap::<Shape2D, Vec<Transform>>::new();

        for (Mesh2D(shape), transform) in sprites {
            sprites_by_shape.entry(shape).or_default().push(transform);
        }

        // shape: (first instance, length)
        let mut sprite_instance_ranges = HashMap::<Shape2D, (u32, u32)>::new();

        let model_matrices =
            sprites_by_shape
                .into_iter()
                .fold(Vec::new(), |mut matrices, (shape, transforms)| {
                    let first_instance = matrices.len() as u32;
                    let instances = transforms.len() as u32;

                    sprite_instance_ranges.insert(shape, (first_instance, instances));

                    matrices.extend(transforms.iter().map(Transform::as_model_matrix));
                    matrices
                });

        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&model_matrices),
        );

        for (shape, (first_instance, count)) in sprite_instance_ranges {
            let mesh = match shape {
                Shape2D::Rectangle => &self.rectangle_mesh,
            };

            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.transform_buffer.slice(..));
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
