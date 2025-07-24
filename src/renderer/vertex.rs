use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
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

/// The instance data per 2D sprite.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SpriteInstance {
    /// The model matrix transformation of the sprite.
    pub transform: Mat4,
    /// The flat color of the sprite.
    pub color: Vec4,
}

impl SpriteVertex {
    /// The vertex layout of per vertex attributes.
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as _,
        step_mode: VertexStepMode::Vertex,
        attributes: &{
            vertex_attr_array![
                0 => Float32x2,
            ]
        },
    };
}

impl SpriteInstance {
    /// The vertex layout of per vertex attributes.
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as _,
        step_mode: VertexStepMode::Instance,
        attributes: &{
            vertex_attr_array![
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
                4 => Float32x4,
                5 => Float32x4,
            ]
        },
    };
}
