use std::sync::Arc;

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

    /// The renderer responsible for rendering the scene and UI.
    renderer: Renderer,

    /// The frame timer which manages the duration of how long each frame took.
    timer: FrameTimer,
}

impl Application {
    /// Initializes the application lifecycle and runs it, acting as the main entry point.
    pub fn run() -> anyhow::Result<()> {
        let event_loop = EventLoop::<Self>::with_user_event().build()?;

        #[cfg(target_family = "wasm")]
        let proxy = event_loop.create_proxy();

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut ApplicationRunner::Initializing {
            #[cfg(target_family = "wasm")]
            proxy: Some(proxy),
        })?;

        Ok(())
    }

    /// Creates a new [`Application`] targetting the given window.
    pub async fn new(window: Arc<Window>) -> Self {
        let timer = FrameTimer::default();

        let renderer = Renderer::new(Arc::clone(&window)).await.unwrap();

        Self {
            window,
            timer,
            renderer,
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => self.resize(size),

            WindowEvent::RedrawRequested => self.render(),

            _ => {}
        }
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
        #[cfg(target_family = "wasm")]
        proxy: Option<EventLoopProxy<Application>>,
    },
    Running(Application),
}

impl ApplicationHandler<Application> for ApplicationRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut attributes = Window::default_attributes();

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
        }

        #[cfg(not(target_family = "wasm"))]
        {
            attributes = attributes.with_inner_size(PhysicalSize::new(1920, 1080));
        }

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        #[cfg(not(target_family = "wasm"))]
        {
            let application = pollster::block_on(Application::new(window));
            *self = Self::Running(application);
        }

        #[cfg(target_family = "wasm")]
        {
            if let Self::Initializing { proxy } = self
                && let Some(proxy) = proxy.take()
            {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(Application::new(window).await).is_ok());
                });
            }
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
