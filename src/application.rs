use std::sync::Arc;

use legion::{Resources, Schedule, World, systems::ParallelRunnable};
#[cfg(target_family = "wasm")]
use winit::event_loop::EventLoopProxy;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{renderer::Renderer, timer::FrameTimer};

pub struct Application {
    /// The main window onto which everything is rendered.
    pub(self) window: Arc<Window>,

    /// The core ECS game world in which all entities live in.
    world: World,
    /// A scheduler for how to excecute systems to act on the `world`.
    scheduler: Schedule,
    /// All resources bound to the `world`.
    resources: Resources,

    /// The renderer responsible for rendering the scene and UI.
    renderer: Renderer,

    /// The frame timer which manages the duration of how long each frame took.
    timer: FrameTimer,
}

/// A builder for an [`Application`],
pub struct ApplicationBuilder {
    /// The systems currently registered to run.
    systems: legion::systems::Builder,
    /// The ECS resources currently bound.
    resources: Option<legion::Resources>,
}

impl ApplicationBuilder {
    /// Registered a system to be run in the default control flow of the app.
    pub fn add_system<T: ParallelRunnable + 'static>(&mut self, system: T) -> &mut Self {
        self.systems.add_system(system);
        self
    }

    /// Finalizes the [`Application`] and runs it.
    pub fn run(&mut self) {
        let event_loop = EventLoop::<Application>::with_user_event().build().unwrap();

        #[cfg(target_family = "wasm")]
        let proxy = event_loop.create_proxy();

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run_app(&mut ApplicationRunner::Initializing {
                systems: Some(self.systems.build()),
                resources: self.resources.take(),
                #[cfg(target_family = "wasm")]
                proxy: Some(proxy),
            })
            .unwrap();
    }
}

impl Application {
    /// Creates a new [`ApplicationBuilder`]; is the main entry point to `ferret`.
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder {
            systems: Schedule::builder(),
            resources: Some(Resources::default()),
        }
    }

    /// Creates a new [`Application`].
    pub(crate) async fn new(
        window: Arc<Window>,
        scheduler: Schedule,
        resources: Resources,
    ) -> Self {
        let timer = FrameTimer::default();

        let renderer = Renderer::new(Arc::clone(&window)).await.unwrap();

        let world = World::default();

        Self {
            window,
            timer,
            renderer,
            world,
            scheduler,
            resources,
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => self.resize(size),

            WindowEvent::RedrawRequested => {
                self.update();
                self.render();
            }

            _ => {}
        }
    }

    fn update(&mut self) {
        self.scheduler.execute(&mut self.world, &mut self.resources);
    }

    fn render(&mut self) {
        self.timer.tick();

        self.window.pre_present_notify();
        self.renderer.render();

        self.window.request_redraw();
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        log::debug!("resizing to new size: {size:?}");
        self.renderer.resize(size);
    }
}

enum ApplicationRunner {
    Initializing {
        systems: Option<Schedule>,
        resources: Option<Resources>,
        #[cfg(target_family = "wasm")]
        proxy: Option<EventLoopProxy<Application>>,
    },
    Running(Application),
}

impl ApplicationHandler<Application> for ApplicationRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();

        let Self::Initializing {
            systems, resources, ..
        } = self
        else {
            return;
        };

        let systems = systems.take().unwrap();
        let resources = resources.take().unwrap();

        #[cfg(target_family = "wasm")]
        {
            use wasm_bindgen::{JsCast, UnwrapThrowExt};
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();

            attributes = attributes.with_canvas(Some(html_canvas_element));

            let window = Arc::new(event_loop.create_window(attributes).unwrap());

            if let Self::Initializing { proxy } = self
                && let Some(proxy) = proxy.take()
            {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(Application::new(window, systems, resources).await)
                            .is_ok()
                    );
                });
            }
        }

        #[cfg(not(target_family = "wasm"))]
        {
            attributes = attributes.with_inner_size(PhysicalSize::new(1920, 1080));

            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            let application = pollster::block_on(Application::new(window, systems, resources));

            *self = Self::Running(application);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Self::Running(app) = self else {
            return;
        };

        app.window_event(event_loop, event);
    }

    fn user_event(&mut self, _: &ActiveEventLoop, mut app: Application) {
        app.resize(app.window.inner_size());
        app.window.request_redraw();

        *self = Self::Running(app);
    }
}
