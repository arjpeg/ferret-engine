use wgpu::*;

use crate::renderer::{shaders::Shaders, vertex::SpriteVertex};

/// All pipelines and bind group layouts used for rendering.
pub struct Pipelines {
    /// The 2D sprite renderer pipeline.
    pub sprite_render_pipeline: RenderPipeline,
}

impl Pipelines {
    /// Creates and initializes all pipelines based on the given shaders.
    pub fn new(device: &Device, shaders: &Shaders, surface_config: &SurfaceConfiguration) -> Self {
        let sprite_render_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Pipelines::sprite_render_pipeline_layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let sprite_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Pipelines::sprite_render_pipeline"),
            layout: Some(&sprite_render_pipeline_layout),
            vertex: VertexState {
                module: shaders.sprite.module(),
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[SpriteVertex::LAYOUT],
            },
            fragment: Some(FragmentState {
                module: shaders.sprite.module(),
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            sprite_render_pipeline,
        }
    }
}
