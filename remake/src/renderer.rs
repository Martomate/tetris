use std::collections::HashMap;

use anyhow::Context;
use wgpu::util::DeviceExt;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{
        FontId, HorizontalAlign, Layout, Section as TextSection, Text, ab_glyph::FontRef,
    },
};

use crate::{
    Game, canvas::Canvas, game::{GameState, Pos}, tile::{Tile, TileRenderer, Vertex}
};

pub mod fonts {
    pub static ARIAL_ROUNDED: &[u8] = include_bytes!("assets/Arial Rounded Bold.ttf");

    pub const ALL_FONTS: [&[u8]; 1] = [ARIAL_ROUNDED];
}

pub struct Renderer {
    fonts: HashMap<&'static [u8], FontId>,
    text_brush: TextBrush<FontRef<'static>>,
    tile_renderer: TileRenderer,
    piece_vertex_buffer: wgpu::Buffer,
    piece_texture_bind_groups: HashMap<char, wgpu::BindGroup>,
    scale_factor: f32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> anyhow::Result<Self> {
        let mut fonts = HashMap::new();
        let mut font_refs = Vec::new();

        for (id, &font_bytes) in fonts::ALL_FONTS.iter().enumerate() {
            fonts.insert(font_bytes, FontId(id));
            font_refs.push(FontRef::try_from_slice(font_bytes).unwrap());
        }

        let text_brush = BrushBuilder::using_fonts(font_refs).build(
            device,
            config.width,
            config.height,
            config.format,
        );

        let tile_renderer = TileRenderer::new(device, config.format);

        let mut piece_texture_bind_groups = HashMap::new();

        static ALL_PIECES: &[(char, &[u8])] = &[
            ('I', include_bytes!("assets/I.png")),
            ('J', include_bytes!("assets/J.png")),
            ('L', include_bytes!("assets/L.png")),
            ('O', include_bytes!("assets/O.png")),
            ('S', include_bytes!("assets/S.png")),
            ('T', include_bytes!("assets/T.png")),
            ('Z', include_bytes!("assets/Z.png")),
        ];

        for &(ch, texture_bytes) in ALL_PIECES {
            let tex = crate::texture::Texture::from_bytes(device, queue, texture_bytes, "piece")
                .context("creating piece texture")?;

            piece_texture_bind_groups.insert(ch, tile_renderer.create_bind_group(device, &tex));
        }

        let tex =
            crate::texture::Texture::from_color(device, queue, [0, 0, 80, 200], "ghost_piece")
                .context("creating ghost piece texture")?;
        piece_texture_bind_groups.insert('G', tile_renderer.create_bind_group(device, &tex));

        let piece_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[0; Vertex::desc().array_stride as usize * 6 * 10 * 20],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            fonts,
            text_brush,
            tile_renderer,
            piece_vertex_buffer,
            piece_texture_bind_groups,
            scale_factor: 1.0, // will be replaced
        })
    }

    pub fn on_resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.text_brush
            .resize_view(width as f32, height as f32, queue);
    }

    pub fn on_scale_factor_changed(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    pub fn render(&mut self, state: &Game, canvas: &Canvas) -> Result<(), wgpu::SurfaceError> {
        self.update_text(state, canvas);

        let frame = canvas.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = canvas
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

            self.render_board(state, canvas, &mut render_pass);
            self.render_text(&mut render_pass);
        }

        canvas.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    fn update_text(&mut self, state: &Game, canvas: &Canvas) {
        let text_sections = self.create_text_sections(state, canvas);
        if let Err(err) = self
            .text_brush
            .queue(&canvas.device, &canvas.queue, text_sections)
        {
            log::error!("Failed to update text: {}", err);
        }
    }

    fn render_text(&mut self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.text_brush.draw(render_pass);
    }

    fn create_text_sections(&self, state: &Game, canvas: &Canvas) -> Vec<TextSection<'static>> {
        let mut sections = Vec::new();

        let cyan_color = [0, 150, 150, 200].map(|c| c as f32 / 255.0);
        let dark_red_color = [150, 0, 0, 255].map(|c| c as f32 / 255.0);

        let big_text = match state.state {
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
                .with_screen_position((canvas.config.width as f32 / 2.0, y_pos * self.scale_factor));

            sections.extend(self.make_text_with_outline(main_section));
        }

        sections
    }

    fn render_board(&mut self, state: &Game, canvas: &Canvas, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.tile_renderer.pipeline);
        render_pass.set_vertex_buffer(0, self.piece_vertex_buffer.slice(..));

        let tile_width = 2.0 / state.board.width as f32;
        let tile_height = 2.0 / state.board.height as f32;

        let mut vertices_written: u32 = 0;

        for (&letter, bind_group) in &self.piece_texture_bind_groups {
            let vertices = self.create_tile_vertices(state, tile_width, tile_height, letter);

            let buffer_offset = vertices_written as u64 * Vertex::desc().array_stride;

            canvas.queue.write_buffer(
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

    fn create_tile_vertices(
        &self,
        state: &Game,
        tile_width: f32,
        tile_height: f32,
        letter: char,
    ) -> Vec<Vertex> {
        let mut spots: Vec<(u8, u8)> = Vec::new();

        if state.state != GameState::Paused {
            for (y, row) in state.board.tiles.iter().enumerate() {
                for (x, &l) in row.iter().enumerate() {
                    if l == letter {
                        spots.push((x as u8, y as u8));
                    }
                }
            }
        }

        if letter == 'G'
            && state.state != GameState::Paused
            && state.state != GameState::GameOver
            && let Some(piece) = state.moving_piece
        {
            let mut piece = piece;
            loop {
                let updated = piece.moved(Pos::new(0, 1));
                if state.piece_collides(updated) {
                    break;
                }
                piece = updated;
            }
            for pos in piece.tiles(&state.shapes) {
                if state.board.contains(pos) {
                    spots.push((pos.x as u8, pos.y as u8));
                }
            }
        }

        if let Some(piece) = state.moving_piece.filter(|p| p.letter == letter) {
            for pos in piece.tiles(&state.shapes) {
                if state.board.contains(pos) {
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
}
