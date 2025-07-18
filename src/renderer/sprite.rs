use glam::{Affine2, Vec2, vec2};
use wgpu::*;

use crate::renderer::{pipelines::Pipelines, vertex::SpriteVertex};

/// The main 2D sprite renderer, responsible for efficiently batching 2D geometry.
pub struct SpriteRenderer {
    /// The vertex buffer for holding the entire render batch.
    vertex_buffer: Buffer,
    /// The index buffer for holding all indices in the batch.
    index_buffer: Buffer,

    /// The actual count of vertices accumulated.
    vertex_count: u32,
    /// The actual count of indices accumulated.
    index_count: u32,
}

/// A 2D sprite to be rendered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    /// The location of the anchor in world space.
    pub position: Vec2,
    /// The rotation of the sprite along the x axis (in radians).
    pub rotation: f32,

    /// The type of primitive geometry the sprite is using.
    pub geometry: Geometry,

    /// The flat color of the sprite.
    pub color: [f32; 3],
}

/// How the sprite is placed in the world relative to its position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The center is used as the anchor.
    Center,
    /// The top left corner is used as the anchor.
    TopLeft,
}

/// A type of primitive geometry object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geometry {
    /// A square.
    Square {
        /// The length of the square's sides.
        length: f32,
        /// Where the anchor of the sprite is.
        anchor: Anchor,
    },
}

impl SpriteRenderer {
    /// The maximum count of vertices count per batch.
    pub const MAX_VERTICES_PER_BATCH: u64 = 100;

    /// Creates a new [`SpriteRenderer`].
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("SpriteRenderer::vertex_buffer"),
            size: Self::MAX_VERTICES_PER_BATCH * size_of::<SpriteVertex>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("SpriteRenderer::index_buffer"),
            size: Self::MAX_VERTICES_PER_BATCH * size_of::<u32>() as u64 * 3,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count: 0,
            index_count: 0,
        }
    }

    /// Submits a batch of sprites to be renderered.
    pub fn submit_sprites(&mut self, queue: &Queue, sprites: Vec<Sprite>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for sprite in sprites {
            let index_offset = vertices.len() as u32;

            vertices.extend(sprite.vertices());
            indices.extend(sprite.geometry.indices().map(|i| i + index_offset));
        }

        assert!(
            vertices.len() < Self::MAX_VERTICES_PER_BATCH as _,
            "vertex count exceeded maximum allocated memory!"
        );

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        self.vertex_count = vertices.len() as _;
        self.index_count = indices.len() as _;
    }

    /// Renders all accumulated sprites to the current render pass.
    pub fn render(&mut self, pipelines: &Pipelines, rpass: &mut RenderPass<'_>) {
        rpass.set_pipeline(&pipelines.sprite_render_pipeline);

        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

impl Sprite {
    /// Converts the sprite into a list of vertices in counter-clockwise order.
    pub(crate) fn vertices(&self) -> impl Iterator<Item = SpriteVertex> {
        self.geometry
            .vertices(self.position, self.rotation)
            .map(|position| SpriteVertex {
                position,
                color: self.color,
            })
    }
}

impl Geometry {
    /// Returns the transformed vertices making up this geometry.
    pub(crate) fn vertices(&self, translation: Vec2, rotation: f32) -> impl Iterator<Item = Vec2> {
        match self {
            Self::Square { length, anchor } => {
                let half_length = length / 2.0;

                let translation = match anchor {
                    Anchor::Center => translation,
                    Anchor::TopLeft => translation + vec2(half_length, -half_length),
                };

                let transform = Affine2::from_scale_angle_translation(
                    vec2(half_length, half_length),
                    rotation,
                    translation,
                );

                [
                    vec2(-1.0, 1.0),
                    vec2(-1.0, -1.0),
                    vec2(1.0, -1.0),
                    vec2(1.0, 1.0),
                ]
                .map(|point| transform.transform_point2(point))
                .into_iter()
            }
        }
    }

    /// Returns the indices in order of how they should be rendered.
    pub(crate) fn indices(&self) -> impl Iterator<Item = u32> {
        match self {
            Self::Square { .. } => [0, 1, 2, 2, 3, 0].into_iter(),
        }
    }
}
