use glam::{Mat4, Quat, Vec3};

/// The transformation of an object in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The translation of the object relative to its center.
    pub translation: Vec3,
    /// The scaling applied to each axis of the object.
    pub scale: Vec3,
    /// The rotation quaternion of the object.
    pub rotation: Quat,
}

impl Transform {
    /// Creates a new [`Transform`] from a given translation.
    pub fn with_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }
    /// Creates a new [`Transform`] from a given scale.
    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Creates a new [`Transform`] from a given rotation.
    pub fn with_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Converts this [`Transform`] into a model matrix.
    pub fn as_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        }
    }
}
