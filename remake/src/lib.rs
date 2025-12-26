mod game;
mod renderer;
mod texture;
mod tile;

use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, TimeDelta, Utc};
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
    game::{Board, Piece, Pos, Shape},
    tile::{Tile, Vertex},
};

pub mod fonts {
    pub static ARIAL_ROUNDED: &[u8] = include_bytes!("assets/Arial Rounded Bold.ttf");

    pub const ALL_FONTS: [&[u8]; 1] = [ARIAL_ROUNDED];
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scale_factor: f32,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,

    board: Board,

    piece_vertex_buffers: HashMap<char, wgpu::Buffer>,
    piece_texture_bind_groups: HashMap<char, wgpu::BindGroup>,
    shapes: HashMap<char, Shape>,

    has_started: bool,
    is_game_over: bool,
    is_paused: bool,
    moving_piece: Option<Piece>,
    next_shape: Option<char>,

    levels_to_win: u8,
    rows_per_level: u8,
    level: u8,
    level_progress: u8,

    time_since_last_move: TimeDelta,
    time_of_last_update: DateTime<Utc>,

    fonts: HashMap<&'static [u8], FontId>,
    text_brush: TextBrush<FontRef<'static>>,
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline =
            renderer::create_render_pipeline(&device, config.format, &[&texture_bind_group_layout]);

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
        let mut piece_vertex_buffers = HashMap::new();

        for &(ch, texture_bytes) in ALL_PIECES {
            let tex = texture::Texture::from_bytes(&device, &queue, texture_bytes, "piece")
                .context("creating piece texture")?;

            piece_texture_bind_groups.insert(
                ch,
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&tex.sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }),
            );

            piece_vertex_buffers.insert(
                ch,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[0; Vertex::desc().array_stride as usize * 6 * 10 * 20],
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

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
            surface,
            device,
            queue,
            config,
            scale_factor: 1.0, // will be replaced
            is_surface_configured: false,
            render_pipeline,
            piece_vertex_buffers,
            window,

            board: Board::new(10, 20),

            piece_texture_bind_groups,
            shapes,

            has_started: false,
            is_game_over: false,
            is_paused: false,
            moving_piece: None,
            next_shape: None,

            levels_to_win: 60,
            rows_per_level: 10,
            level: 0,
            level_progress: 0,
            time_since_last_move: TimeDelta::zero(),
            time_of_last_update: Utc::now(),

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

            render_pass.set_pipeline(&self.render_pipeline);
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

        let big_text = if !self.has_started {
            Some((("Press\nSPACE", cyan_color, 60.0), 160.0))
        } else if self.is_game_over {
            Some((("Game Over", dark_red_color, 60.0), 260.0))
        } else if self.is_paused {
            Some((("Press P", cyan_color, 60.0), 160.0))
        } else {
            None
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
        let tw = 2.0 / self.board.width as f32;
        let th = 2.0 / self.board.height as f32;

        for (letter, bind_group) in &self.piece_texture_bind_groups {
            let mut spots: Vec<(u8, u8)> = Vec::new();

            if !self.is_paused || self.is_game_over {
                for (y, row) in self.board.tiles.iter().enumerate() {
                    for (x, l) in row.iter().enumerate() {
                        if l == letter {
                            spots.push((x as u8, y as u8));
                        }
                    }
                }
            }

            if let Some(piece) = self.moving_piece.filter(|p| p.letter == *letter) {
                for pos in self.shapes[letter].rotated(piece.rotation).at(piece.origin) {
                    if self.board.contains(pos) {
                        spots.push((pos.x as u8, pos.y as u8));
                    }
                }
            }

            let tiles = spots
                .iter()
                .map(|&(x, y)| Tile::new(tw, th).at(tw * x as f32 - 1.0, 1.0 - th * (y + 1) as f32))
                .collect::<Vec<_>>();

            let vertices = tiles.iter().flat_map(|t| t.vertices).collect::<Vec<_>>();

            self.queue.write_buffer(
                &self.piece_vertex_buffers[letter],
                0,
                bytemuck::cast_slice(&vertices),
            );

            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.piece_vertex_buffers[letter].slice(..));

            let buffer_len = spots.len() as u32 * 6;
            render_pass.draw(0..buffer_len, 0..1);
        }
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

        let progress = self.level as f32 / self.levels_to_win as f32;
        let speed_up = (progress * PI / 2.0).sin();
        TimeDelta::milliseconds(((1.0 - speed_up) * 800.0) as i64)
    }

    fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => {
                self.is_paused = !self.is_paused;
            }
            (KeyCode::KeyP, true) => {
                self.is_paused = !self.is_paused;
            }
            (KeyCode::Space, true) => {
                if !self.has_started {
                    self.has_started = true;
                    self.is_paused = false;
                }
            }
            (KeyCode::ArrowUp, true) => {
                if let Some(piece) = self.moving_piece {
                    let updated = Piece {
                        rotation: (piece.rotation + 1) % 4,
                        ..piece
                    };
                    if !self.piece_collides(updated) {
                        self.moving_piece = Some(updated);
                    }
                }
            }
            (KeyCode::ArrowLeft, true) => {
                if let Some(piece) = self.moving_piece {
                    let updated = Piece {
                        origin: piece.origin + Pos::new(-1, 0),
                        ..piece
                    };
                    if !self.piece_collides(updated) {
                        self.moving_piece = Some(updated);
                    }
                }
            }
            (KeyCode::ArrowDown, true) => {
                if let Some(piece) = self.moving_piece {
                    let updated = Piece {
                        origin: piece.origin + Pos::new(0, 1),
                        ..piece
                    };
                    if !self.piece_collides(updated) {
                        self.moving_piece = Some(updated);
                    }
                }
            }
            (KeyCode::ArrowRight, true) => {
                if let Some(piece) = self.moving_piece {
                    let updated = Piece {
                        origin: piece.origin + Pos::new(1, 0),
                        ..piece
                    };
                    if !self.piece_collides(updated) {
                        self.moving_piece = Some(updated);
                    }
                }
            }
            (KeyCode::KeyD, true) => {
                while let Some(piece) = self.moving_piece {
                    let updated = Piece {
                        origin: piece.origin + Pos::new(0, 1),
                        ..piece
                    };
                    if !self.piece_collides(updated) {
                        self.moving_piece = Some(updated);
                    } else {
                        self.handle_dropped_piece();
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_dropped_piece(&mut self) {
        if let Some(piece) = self.moving_piece.take() {
            for pos in self.shapes[&piece.letter]
                .rotated(piece.rotation)
                .at(piece.origin)
            {
                self.board.set_tile(pos, piece.letter);
            }
        }

        self.remove_full_rows();
    }

    fn remove_full_rows(&mut self) {
        let mut removed_rows = 0;
        for y in (0..self.board.height).rev() {
            let mut full_row = true;
            for x in 0..self.board.width {
                if self.board.get_tile(Pos::new(x as i8, y as i8)).is_none() {
                    full_row = false;
                    break;
                }
            }
            if full_row {
                removed_rows += 1;
            } else {
                for x in 0..self.board.width {
                    if let Some(tile) = self.board.get_tile(Pos::new(x as i8, y as i8)) {
                        self.board
                            .set_tile(Pos::new(x as i8, (y + removed_rows) as i8), tile);
                    }
                }
            }
            if removed_rows > 0 {
                for x in 0..self.board.width {
                    self.board.clear_tile(Pos::new(x as i8, y as i8));
                }
            }
        }
        self.level_progress += removed_rows;
    }

    fn update(&mut self) {
        let now = Utc::now();
        let time_passed = now.signed_duration_since(self.time_of_last_update);
        self.time_of_last_update = now;

        if !self.has_started {
            return;
        }
        if self.is_game_over {
            return;
        }
        if self.is_paused {
            return;
        }

        self.time_since_last_move += time_passed;

        if self.level_progress >= self.rows_per_level {
            self.level_progress -= self.rows_per_level;
            self.level += 1;
        }
        if self.time_since_last_move >= self.time_between_moves() {
            self.time_since_last_move -= self.time_between_moves();

            if let Some(piece) = self.moving_piece {
                let updated = Piece {
                    origin: piece.origin + Pos::new(0, 1),
                    ..piece
                };
                if !self.piece_collides(updated) {
                    self.moving_piece = Some(updated);
                } else {
                    self.handle_dropped_piece();
                }
            }
        }
        if self.moving_piece.is_none()
            && let Some(letter) = self.next_shape.take()
        {
            let piece = Piece {
                letter,
                rotation: 0,
                origin: Pos { x: 4, y: 1 },
            };
            self.moving_piece = Some(piece);
            self.time_since_last_move = TimeDelta::zero();

            if self.piece_collides(piece) {
                self.is_game_over = true;
            }
        }
        if self.next_shape.is_none() {
            self.next_shape = Some(random_shape(
                &self.shapes.keys().cloned().collect::<Vec<char>>(),
            ));
        }
    }

    fn piece_collides(&self, piece: Piece) -> bool {
        let positions = self.shapes[&piece.letter]
            .rotated(piece.rotation)
            .at(piece.origin);

        positions
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
                state.update();
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
                if !focused && state.has_started {
                    state.is_paused = true;
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
