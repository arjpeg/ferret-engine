use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use wgpu::{VertexBufferLayout, VertexStepMode, vertex_attr_array};

/// A vertex of a 2D sprite triangle.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SpriteVertex {
    /// The 2D world space coordinates of the vertex.
    pub position: Vec2,
    /// The flat color of the vertex.
    pub color: [f32; 3],
}

impl SpriteVertex {
    /// The vertex layout of per vertex attributes.
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as _,
        step_mode: VertexStepMode::Vertex,
        attributes: &{
            vertex_attr_array![
                0 => Float32x2,
                1 => Float32x3
            ]
        },
    };
}
