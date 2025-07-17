use std::sync::Arc;

use anyhow::Context;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{pipelines::Pipelines, shaders::Shaders, sprite::SpriteRenderer};

mod pipelines;
mod shaders;
mod sprite;
mod vertex;

/// All state that is required for drawing a full scene and UI.
pub struct Renderer {
    device: Device,
    queue: Queue,

    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,

    shaders: Shaders,

    pipelines: Pipelines,

    sprite_renderer: SpriteRenderer,
}

impl Renderer {
    /// Creates a new [`Renderer`] targetting the given window as the rendering surface.
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::from_env().unwrap_or(Backends::all()),
            flags: InstanceFlags::all(),
            backend_options: BackendOptions::default(),
        });

        let surface = instance
            .create_surface(window)
            .context("while creating the rendering surface")?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("while requesting adapter")?;

        log::debug!("selected adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Renderer::request_device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("while requesting rendering device")?;

        let surface_config = Self::get_surface_configuration(&surface, &adapter, size);
        surface.configure(&device, &surface_config);

        let shaders = Shaders::new(&device);

        let pipelines = Pipelines::new(&device, &shaders, &surface_config);

        let sprite_renderer = SpriteRenderer::new(&device);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            shaders,
            pipelines,
            sprite_renderer,
        })
    }

    /// Returns an appropriate [`SurfaceConfiguration`] for rendering.
    fn get_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        size: PhysicalSize<u32>,
    ) -> SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);

        let format = surface_caps
            .formats
            .iter()
            .cloned()
            .find(TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);

        let PhysicalSize { width, height } = size;

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        }
    }

    /// Resizes the target surface to match the new size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Renders the entire scene and all UI.
    pub fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(tex) => tex,

            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                let SurfaceConfiguration { width, height, .. } = self.surface_config;
                self.resize(PhysicalSize { width, height });
                return;
            }

            Err(e) => panic!("unhandled surface error: {e}"),
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Renderer::main_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            use glam::*;
            use sprite::*;

            self.sprite_renderer.submit_sprites(
                &self.queue,
                vec![Sprite {
                    position: vec2(0.0, 0.0),
                    rotation: 0.0,
                    geometry: Geometry::Square {
                        length: 0.01,
                        anchor: Anchor::Center,
                    },
                    color: [0.3, 0.5, 0.6],
                }],
            );

            self.sprite_renderer.render(&self.pipelines, &mut pass);
        }

        self.queue.submit([encoder.finish()]);
        output.present();
    }
}
