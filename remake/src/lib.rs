mod game;
mod texture;

use std::{collections::HashMap, sync::Arc};

use rand::Rng;
use wgpu::util::DeviceExt;
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

use crate::game::{Board, Piece, Pos, Shape};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,
    clear_color: wgpu::Color,

    board: Board,

    piece_vertex_buffers: HashMap<char, wgpu::Buffer>,
    piece_textures: HashMap<char, texture::Texture>,
    piece_texture_bind_groups: HashMap<char, wgpu::BindGroup>,
    shapes: HashMap<char, Shape>,

    is_game_over: bool,
    moving_piece: Option<Piece>,
    next_shape: Option<char>,
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

        let surface = instance.create_surface(window.clone()).unwrap();

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

        static ALL_PIECES: &[(char, &[u8])] = &[
            ('I', include_bytes!("assets/I.png")),
            ('J', include_bytes!("assets/J.png")),
            ('L', include_bytes!("assets/L.png")),
            ('O', include_bytes!("assets/O.png")),
            ('S', include_bytes!("assets/S.png")),
            ('T', include_bytes!("assets/T.png")),
            ('Z', include_bytes!("assets/Z.png")),
        ];

        let piece_textures = ALL_PIECES
            .iter()
            .map(|(ch, texture_bytes)| {
                (
                    *ch,
                    texture::Texture::from_bytes(&device, &queue, texture_bytes, "piece").unwrap(),
                )
            })
            .collect::<HashMap<char, texture::Texture>>();

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

        let piece_texture_bind_groups = piece_textures
            .iter()
            .map(|(letter, tex)| {
                (
                    *letter,
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
                )
            })
            .collect();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let piece_vertex_buffers = piece_textures
            .keys()
            .map(|l| {
                (
                    *l,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: &[0; Vertex::desc().array_stride as usize * 10 * 20],
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    }),
                )
            })
            .collect::<HashMap<char, wgpu::Buffer>>();

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
            is_surface_configured: false,
            render_pipeline,
            piece_vertex_buffers,
            window,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },

            board: Board::new(10, 20),

            piece_textures,
            piece_texture_bind_groups,
            shapes,

            is_game_over: false,
            moving_piece: None,
            next_shape: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            let tw = 2.0 / self.board.width as f32;
            let th = 2.0 / self.board.height as f32;

            for (letter, bind_group) in &self.piece_texture_bind_groups {
                let mut spots: Vec<(u8, u8)> = Vec::new();
                for (y, row) in self.board.tiles.iter().enumerate() {
                    for (x, l) in row.iter().enumerate() {
                        if l == letter {
                            spots.push((x as u8, y as u8));
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
                    .map(|&(x, y)| {
                        Tile::new(tw, th).at(tw * x as f32 - 1.0, 1.0 - th * (y + 1) as f32)
                    })
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

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
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
                        self.board.set_tile(Pos::new(x as i8, (y + removed_rows) as i8), tile);
                    }
                }
            }
            if removed_rows > 0 {
                for x in 0..self.board.width {
                    self.board.clear_tile(Pos::new(x as i8, y as i8));
                }
            }
        }
    }
    
    fn handle_mouse_moved(&mut self, x: f64, y: f64) {}

    fn update(&mut self) {
        if self.is_game_over {
            return;
        }
        if self.moving_piece.is_none() && self.next_shape.is_some() {
            let piece = Piece {
                letter: self.next_shape.take().unwrap(),
                rotation: 0,
                origin: Pos { x: 4, y: 1 },
            };
            self.moving_piece = Some(piece);

            if self.piece_collides(piece) {
                self.is_game_over = true;
            }
        }
        if self.next_shape.is_none() {
            self.next_shape = Some(random_shape(&self.shapes.keys().cloned().collect::<Vec<char>>()));
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

struct Tile {
    vertices: [Vertex; 6],
}

impl Tile {
    fn new(w: f32, h: f32) -> Tile {
        Tile {
            vertices: [
                Vertex {
                    position: [0.0, h, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [w, h, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [w, h, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [w, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
        }
    }

    fn at(mut self, x: f32, y: f32) -> Self {
        for v in &mut self.vertices {
            v.position[0] += x;
            v.position[1] += y;
        }
        self
    }
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
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
                if focused {
                    log::info!("Gained focus!");
                } else {
                    log::info!("Lost focus!");
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
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved { position, .. } => {
                state.handle_mouse_moved(position.x, position.y)
            }
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
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
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
