use std::borrow::Cow;

use resource::{Resource, resource_str};
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

/// All shaders used for rendering.
pub struct Shaders {
    /// The shader used for 2D sprite rendering.
    pub sprite: Shader,
}

/// A reloadable and compiler shader.
pub struct Shader {
    /// The internal resource used to track the shader's source.
    resource: Resource<str>,
    /// The compiled shader module.
    module: ShaderModule,
}

impl Shaders {
    /// Creates and compiles all shader modules needed for rendering.
    pub fn new(device: &Device) -> Self {
        let sprite = Shader::new(
            device,
            resource_str!("assets/shaders/sprite.wgsl"),
            "Shaders::sprite",
        );

        Self { sprite }
    }
}

impl Shader {
    /// Creates a new [`Shader`] pointing at the given resource.
    pub fn new(device: &Device, resource: Resource<str>, label: &str) -> Self {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(label),
            source: ShaderSource::Wgsl(Cow::Borrowed(&*resource)),
        });

        Self { resource, module }
    }

    /// Returns the compiled module describing this shader.
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }
}
