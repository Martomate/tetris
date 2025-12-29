mod canvas;
mod game;
mod renderer;
mod texture;
mod tile;
mod time;

use std::sync::Arc;

use chrono::Utc;
use winit::{
    application::ApplicationHandler,
    dpi,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{canvas::Canvas, game::Game, renderer::Renderer, time::Clock};

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Canvas>>,
    game: Option<Game>,
    renderer: Option<Renderer>,
    canvas: Option<Canvas>,
    clock: Clock,
}

impl App {
    pub fn new(#[allow(unused)] event_loop: &EventLoop<Canvas>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            game: None,
            renderer: None,
            canvas: None,
            clock: Clock::now(),
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<Canvas> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes =
            Window::default_attributes().with_inner_size(dpi::LogicalSize::new(320, 640));

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let canvas = pollster::block_on(Canvas::new(window)).unwrap();
            self.game = Some(Game::default());
            self.renderer =
                Some(Renderer::new(&canvas.device, &canvas.queue, &canvas.config).unwrap());
            self.canvas = Some(canvas);
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                Canvas::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut canvas: Canvas) {
        #[cfg(target_arch = "wasm32")]
        {
            canvas.window.request_redraw();
            let width = canvas.window.inner_size().width;
            let height = canvas.window.inner_size().height;
            if width > 0 && height > 0 {
                canvas.resize(width, height);
            }
        }
        self.game = Some(Game::default());
        self.renderer = Some(Renderer::new(&canvas.device, &canvas.queue, &canvas.config).unwrap());
        self.canvas = Some(canvas);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(canvas) = &mut self.canvas else {
            return;
        };
        let Some(game) = &mut self.game else {
            return;
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    canvas.resize(size.width, size.height);
                    renderer.on_resize(&canvas.queue, size.width, size.height);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                renderer.on_scale_factor_changed(scale_factor as f32);
            }
            WindowEvent::RedrawRequested => {
                let time_passed = self.clock.update(Utc::now());
                game.update(time_passed);
                canvas.window.request_redraw();
                if !canvas.is_surface_configured {
                    return;
                }
                match renderer.render(game, canvas) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = canvas.window.inner_size();
                        if size.width > 0 && size.height > 0 {
                            canvas.resize(size.width, size.height);
                            renderer.on_resize(&canvas.queue, size.width, size.height);
                        }
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                game.on_focus_changed(focused);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => game.handle_key(code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
