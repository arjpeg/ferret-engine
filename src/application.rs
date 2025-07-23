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
    /// A scheduler for how to excecute systems to act on the `world` during the update cycle.
    update_schedule: Schedule,
    /// All resources bound to the `world`.
    resources: Resources,

    /// The renderer responsible for rendering the scene and UI.
    renderer: Renderer,
}

impl Application {
    /// Creates a new [`ApplicationBuilder`]; is the main entry point to `ferret`.
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    /// Creates a new [`Application`].
    pub(crate) async fn new(
        window: Arc<Window>,
        mut startup_schedule: Schedule,
        update_schedule: Schedule,
        mut resources: Resources,
    ) -> Self {
        let renderer = Renderer::new(Arc::clone(&window)).await.unwrap();

        let mut world = World::default();
        resources.insert(FrameTimer::default());

        startup_schedule.execute(&mut world, &mut resources);

        Self {
            window,
            renderer,
            world,
            update_schedule,
            resources,
        }
    }

    /// Handles an incoming [`WindowEvent`]
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

    /// Runs the main update cycle of the application.
    fn update(&mut self) {
        if let Some(mut timer) = self.resources.get_mut::<FrameTimer>() {
            timer.tick();
        }

        self.update_schedule
            .execute(&mut self.world, &mut self.resources);
    }

    /// Renders the game world and all UI.
    fn render(&mut self) {
        self.window.pre_present_notify();
        self.renderer.render();

        self.window.request_redraw();
    }

    /// Resizes the internal state of the application to match the window's size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        log::debug!("resizing to new size: {size:?}");
        self.renderer.resize(size);
    }
}

enum ApplicationRunner {
    Initializing {
        /// The systems added to to the ECS update loop.
        update_schedule: Option<Schedule>,
        /// The systems to be run during initialization.
        startup_schedule: Option<Schedule>,
        /// The custom resources added to to the ECS world.
        resources: Option<Resources>,
        /// A proxy to manage the async inititalization on the web.
        #[cfg(target_family = "wasm")]
        proxy: Option<EventLoopProxy<Application>>,
    },
    Running(Application),
}

impl ApplicationHandler<Application> for ApplicationRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();

        let Self::Initializing {
            update_schedule,
            startup_schedule,
            resources,
            ..
        } = self
        else {
            return;
        };

        let update_systems = update_schedule.take().unwrap();
        let startup_systems = startup_schedule.take().unwrap();
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
                    let app =
                        Application::new(window, startup_systems, update_systems, resources).await;

                    assert!(proxy.send_event(app).is_ok());
                });
            }
        }

        #[cfg(not(target_family = "wasm"))]
        {
            attributes = attributes.with_inner_size(PhysicalSize::new(1920, 1080));

            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            let application = pollster::block_on(Application::new(
                window,
                startup_systems,
                update_systems,
                resources,
            ));

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

/// A builder for an [`Application`],
pub struct ApplicationBuilder {
    /// The systems currently registered to run per frame.
    update_systems: legion::systems::Builder,
    /// The systems currently registered to run once at application startup.
    startup_systems: legion::systems::Builder,

    /// The ECS resources currently bound.
    resources: Option<legion::Resources>,
}

impl ApplicationBuilder {
    /// Registers a system to be run once at application initialization.
    pub fn add_startup_system<T: ParallelRunnable + 'static>(&mut self, system: T) -> &mut Self {
        self.startup_systems.add_system(system);
        self
    }

    /// Registers a system to be run in the update cycle of the app.
    pub fn add_update_system<T: ParallelRunnable + 'static>(&mut self, system: T) -> &mut Self {
        self.update_systems.add_system(system);
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
                startup_schedule: Some(self.startup_systems.build()),
                update_schedule: Some(self.update_systems.build()),
                resources: self.resources.take(),
                #[cfg(target_family = "wasm")]
                proxy: Some(proxy),
            })
            .unwrap();
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        use legion::systems::Builder;

        Self {
            update_systems: Builder::default(),
            startup_systems: Builder::default(),
            resources: Some(Resources::default()),
        }
    }
}
