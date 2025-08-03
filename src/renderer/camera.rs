use glam::Mat4;

use crate::{
    ecs::World,
    prelude::{Material2D, Mesh2D, Transform},
};

/// Represents a orthographic camera in 2D from which all 2D sprites will be rendered.
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// The size of the camera's view width in meters from the center of the screen.
    pub half_width: f32,
}

impl Camera2D {
    /// Calculates the orthographic projection matrix based on the current camera and window state.
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let half_height = self.half_width / aspect_ratio;
        Mat4::orthographic_rh(
            -self.half_width,
            self.half_width,
            -half_height,
            half_height,
            -1.0,
            1.0,
        )
    }

    /// Calculates the view matrix based on the given camera transform.
    pub fn view_matrix(transform: &Transform) -> Mat4 {
        transform.as_model_matrix().inverse()
    }

    /// Extracts the entities to be rendered from this camera.
    pub fn extract_entities(&self, world: &World) -> Vec<(Mesh2D, Material2D, Transform)> {
        world
            .query::<(&Mesh2D, &Material2D, &Transform)>()
            .iter()
            .map(|(_, (mesh, material, transform))| (*mesh, *material, *transform))
            .collect()
    }
}
