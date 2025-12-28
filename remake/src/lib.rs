mod game;
mod texture;
mod tile;
mod time;

use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use chrono::{TimeDelta, Utc};
use rand::Rng;
use wgpu::util::DeviceExt;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{
        FontId, HorizontalAlign, Layout, Section as TextSection, Text, ab_glyph::FontRef,
    },
};
use winit::{
    application::ApplicationHandler,
    dpi,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
    game::{Board, GameProgress, GameState, Piece, Pos, Shape},
    tile::{Tile, TileRenderer, Vertex},
    time::{Clock, Timer},
};

pub mod fonts {
    pub static ARIAL_ROUNDED: &[u8] = include_bytes!("assets/Arial Rounded Bold.ttf");

    pub const ALL_FONTS: [&[u8]; 1] = [ARIAL_ROUNDED];
}

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scale_factor: f32,
    is_surface_configured: bool,
    clock: Clock,

    fonts: HashMap<&'static [u8], FontId>,
    text_brush: TextBrush<FontRef<'static>>,
    tile_renderer: TileRenderer,
    piece_vertex_buffer: wgpu::Buffer,
    piece_texture_bind_groups: HashMap<char, wgpu::BindGroup>,

    board: Board,
    state: GameState,
    shapes: HashMap<char, Shape>,
    moving_piece_timer: Timer,
    moving_piece: Option<Piece>,
    next_shape: Option<char>,
    progress: GameProgress,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .context("creating surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut fonts = HashMap::new();
        let mut font_refs = Vec::new();

        for (id, &font_bytes) in fonts::ALL_FONTS.iter().enumerate() {
            fonts.insert(font_bytes, FontId(id));
            font_refs.push(FontRef::try_from_slice(font_bytes).unwrap());
        }

        let text_brush = BrushBuilder::using_fonts(font_refs).build(
            &device,
            config.width,
            config.height,
            config.format,
        );

        let tile_renderer = TileRenderer::new(&device, config.format);

        static ALL_PIECES: &[(char, &[u8])] = &[
            ('I', include_bytes!("assets/I.png")),
            ('J', include_bytes!("assets/J.png")),
            ('L', include_bytes!("assets/L.png")),
            ('O', include_bytes!("assets/O.png")),
            ('S', include_bytes!("assets/S.png")),
            ('T', include_bytes!("assets/T.png")),
            ('Z', include_bytes!("assets/Z.png")),
        ];

        let mut piece_texture_bind_groups = HashMap::new();

        for &(ch, texture_bytes) in ALL_PIECES {
            let tex = texture::Texture::from_bytes(&device, &queue, texture_bytes, "piece")
                .context("creating piece texture")?;

            piece_texture_bind_groups.insert(ch, tile_renderer.create_bind_group(&device, &tex));
        }

        let tex = texture::Texture::from_color(&device, &queue, [0, 0, 80, 200], "ghost_piece")
            .context("creating ghost piece texture")?;
        piece_texture_bind_groups.insert('G', tile_renderer.create_bind_group(&device, &tex));

        let piece_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[0; Vertex::desc().array_stride as usize * 6 * 10 * 20],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let shapes: HashMap<char, Shape> = [
            ('O', [(0, -1), (0, 0), (1, 0), (1, -1)]),
            ('I', [(0, -1), (0, 0), (0, 1), (0, 2)]),
            ('J', [(1, -1), (1, 0), (1, 1), (0, 1)]),
            ('L', [(0, -1), (0, 0), (0, 1), (1, 1)]),
            ('Z', [(1, -1), (1, 0), (0, 0), (0, 1)]),
            ('S', [(0, -1), (0, 0), (1, 0), (1, 1)]),
            ('T', [(0, -1), (0, 0), (0, 1), (1, 0)]),
        ]
        .map(|(l, offsets)| (l, Shape::new(offsets.map(|(dx, dy)| Pos::new(dx, dy)))))
        .into_iter()
        .collect();

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            scale_factor: 1.0, // will be replaced
            is_surface_configured: false,

            tile_renderer,
            piece_vertex_buffer,
            piece_texture_bind_groups,
            shapes,

            board: Board::new(10, 20),
            state: GameState::NotStarted,
            moving_piece: None,
            next_shape: None,

            progress: GameProgress::new(60, 10),
            moving_piece_timer: Timer::new(),
            clock: Clock::now(),

            fonts,
            text_brush,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.text_brush.resize_view(
                self.config.width as f32,
                self.config.height as f32,
                &self.queue,
            );
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        self.update_text();

        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.render_board(&mut render_pass);
            self.render_text(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    fn update_text(&mut self) {
        let text_sections = self.create_text_sections();
        if let Err(err) = self
            .text_brush
            .queue(&self.device, &self.queue, text_sections)
        {
            log::error!("Failed to update text: {}", err);
        }
    }

    fn render_text(&mut self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.text_brush.draw(render_pass);
    }

    fn create_text_sections(&self) -> Vec<TextSection<'static>> {
        let mut sections = Vec::new();

        let cyan_color = [0, 150, 150, 200].map(|c| c as f32 / 255.0);
        let dark_red_color = [150, 0, 0, 255].map(|c| c as f32 / 255.0);

        let big_text = match self.state {
            GameState::NotStarted => Some((("Press\nSPACE", cyan_color, 60.0), 160.0)),
            GameState::GameOver => Some((("Game Over", dark_red_color, 60.0), 260.0)),
            GameState::Paused => Some((("Press P", cyan_color, 60.0), 160.0)),
            GameState::Running => None,
        };

        if let Some(((text, color, scale), y_pos)) = big_text {
            let main_section = TextSection::default()
                .add_text(
                    Text::new(text)
                        .with_color(color)
                        .with_scale(scale * self.scale_factor)
                        .with_font_id(self.fonts[fonts::ARIAL_ROUNDED]),
                )
                .with_layout(Layout::default().h_align(HorizontalAlign::Center))
                .with_screen_position((self.config.width as f32 / 2.0, y_pos * self.scale_factor));

            sections.extend(self.make_text_with_outline(main_section));
        }

        sections
    }

    fn render_board(&mut self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.tile_renderer.pipeline);
        render_pass.set_vertex_buffer(0, self.piece_vertex_buffer.slice(..));

        let tile_width = 2.0 / self.board.width as f32;
        let tile_height = 2.0 / self.board.height as f32;

        let mut vertices_written: u32 = 0;

        for (&letter, bind_group) in &self.piece_texture_bind_groups {
            let vertices = self.create_tile_vertices(tile_width, tile_height, letter);

            let buffer_offset = vertices_written as u64 * Vertex::desc().array_stride;

            self.queue.write_buffer(
                &self.piece_vertex_buffer,
                buffer_offset,
                bytemuck::cast_slice(&vertices),
            );

            let num_vertices = vertices.len() as u32;

            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(vertices_written..vertices_written + num_vertices, 0..1);

            vertices_written += num_vertices;
        }
    }

    fn create_tile_vertices(&self, tile_width: f32, tile_height: f32, letter: char) -> Vec<Vertex> {
        let mut spots: Vec<(u8, u8)> = Vec::new();

        if self.state != GameState::Paused {
            for (y, row) in self.board.tiles.iter().enumerate() {
                for (x, &l) in row.iter().enumerate() {
                    if l == letter {
                        spots.push((x as u8, y as u8));
                    }
                }
            }
        }

        if letter == 'G'
            && self.state != GameState::Paused
            && self.state != GameState::GameOver
            && let Some(piece) = self.moving_piece
        {
            let mut piece = piece;
            loop {
                let updated = piece.moved(Pos::new(0, 1));
                if self.piece_collides(updated) {
                    break;
                }
                piece = updated;
            }
            for pos in piece.tiles(&self.shapes) {
                if self.board.contains(pos) {
                    spots.push((pos.x as u8, pos.y as u8));
                }
            }
        }

        if let Some(piece) = self.moving_piece.filter(|p| p.letter == letter) {
            for pos in piece.tiles(&self.shapes) {
                if self.board.contains(pos) {
                    spots.push((pos.x as u8, pos.y as u8));
                }
            }
        }

        let tiles = spots
            .iter()
            .map(|&(x, y)| {
                let tx = tile_width * x as f32 - 1.0;
                let ty = 1.0 - tile_height * (y + 1) as f32;
                Tile::new(tile_width, tile_height).at(tx, ty)
            })
            .collect::<Vec<_>>();

        tiles.iter().flat_map(|t| t.vertices).collect::<Vec<_>>()
    }

    fn make_text_with_outline<'f>(&self, section: TextSection<'f>) -> Vec<TextSection<'f>> {
        let mut res = Vec::new();

        let d: f32 = 2.0;

        let (x, y) = section.screen_position;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 || dy != 0 {
                    let text = section
                        .text
                        .iter()
                        .map(|t| t.with_color([0.0, 0.0, 0.0, 1.0]))
                        .collect();

                    res.push(
                        section
                            .clone()
                            .with_text(text)
                            .with_screen_position((x + dx as f32 * d, y + dy as f32 * d)),
                    );
                }
            }
        }
        res.push(section);

        res
    }

    /// 800 ms (level 0) to 0 ms (max level), reducing faster in the beginning
    fn time_between_moves(&self) -> TimeDelta {
        use std::f32::consts::PI;

        let progress = self.progress.level as f32 / self.progress.levels_to_win as f32;
        let speed_up = (progress * PI / 2.0).sin();
        TimeDelta::milliseconds(((1.0 - speed_up) * 800.0) as i64)
    }

    fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => {
                self.state = match self.state {
                    GameState::Running => GameState::Paused,
                    GameState::Paused => GameState::Running,
                    state => state,
                };
            }
            (KeyCode::KeyP, true) => {
                self.state = match self.state {
                    GameState::Running => GameState::Paused,
                    GameState::Paused => GameState::Running,
                    state => state,
                };
            }
            (KeyCode::Space, true) => {
                if self.state == GameState::NotStarted {
                    self.state = GameState::Running;
                }
            }
            (KeyCode::ArrowUp, true) => {
                if let Some(piece) = self.try_update_moving_piece(|p| p.rotated_cw()) {
                    self.moving_piece = Some(piece);
                }
            }
            (KeyCode::ArrowLeft, true) => {
                if let Some(piece) = self.try_update_moving_piece(|p| p.moved(Pos::new(-1, 0))) {
                    self.moving_piece = Some(piece);
                }
            }
            (KeyCode::ArrowDown, true) => {
                if let Some(piece) = self.try_update_moving_piece(|p| p.moved(Pos::new(0, 1))) {
                    self.moving_piece = Some(piece);
                }
            }
            (KeyCode::ArrowRight, true) => {
                if let Some(piece) = self.try_update_moving_piece(|p| p.moved(Pos::new(1, 0))) {
                    self.moving_piece = Some(piece);
                }
            }
            (KeyCode::KeyD, true) => {
                while let Some(piece) = self.try_update_moving_piece(|p| p.moved(Pos::new(0, 1))) {
                    self.moving_piece = Some(piece);
                }

                self.handle_dropped_piece();
            }
            _ => {}
        }
    }

    fn try_update_moving_piece(&mut self, update_fn: impl FnOnce(Piece) -> Piece) -> Option<Piece> {
        if let Some(piece) = self.moving_piece {
            let updated = update_fn(piece);
            if !self.piece_collides(updated) {
                return Some(updated);
            }
        }
        None
    }

    fn handle_dropped_piece(&mut self) {
        if let Some(piece) = self.moving_piece.take() {
            for pos in piece.tiles(&self.shapes) {
                self.board.set_tile(pos, piece.letter);
            }
        }

        self.progress.add_rows(self.board.remove_full_rows());
    }

    fn update(&mut self, time_passed: TimeDelta) {
        if self.state != GameState::Running {
            return;
        }

        self.moving_piece_timer.advance(time_passed);
        while self.moving_piece_timer.tick(self.time_between_moves()) {
            if let Some(piece) = self.try_update_moving_piece(|p| p.moved(Pos::new(0, 1))) {
                self.moving_piece = Some(piece);
            } else {
                self.handle_dropped_piece();
            }
        }
        if self.moving_piece.is_none()
            && let Some(letter) = self.next_shape.take()
        {
            let piece = Piece::new(letter, 0, Pos { x: 4, y: 1 });
            self.moving_piece = Some(piece);
            self.moving_piece_timer.reset();

            if self.piece_collides(piece) {
                self.state = GameState::GameOver;
            }
        }
        if self.next_shape.is_none() {
            self.next_shape = Some(random_shape(
                &self.shapes.keys().cloned().collect::<Vec<char>>(),
            ));
        }
    }

    fn piece_collides(&self, piece: Piece) -> bool {
        piece
            .tiles(&self.shapes)
            .iter()
            .any(|&pos| !self.board.contains(pos) || self.board.get_tile(pos).is_some())
    }
}

fn random_shape(values: &[char]) -> char {
    values[rand::rng().random_range(0..values.len())]
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}

impl App {
    pub fn new(#[allow(unused)] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<State> for App {
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
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
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
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                state.scale_factor = scale_factor as f32;
            }
            WindowEvent::RedrawRequested => {
                let time_passed = state.clock.update(Utc::now());
                state.update(time_passed);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                if !focused && state.state != GameState::NotStarted {
                    state.state = GameState::Paused;
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(code, key_state.is_pressed()),
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
