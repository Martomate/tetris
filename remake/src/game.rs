use std::{
    collections::HashMap,
    ops::{Add, Index},
};

use chrono::TimeDelta;
use rand::Rng;
use winit::keyboard::KeyCode;

use crate::time::Timer;

pub struct Game {
    pub board: Board,
    pub state: GameState,
    pub shapes: HashMap<char, Shape>,
    pub moving_piece_timer: Timer,
    pub moving_piece: Option<Piece>,
    pub next_shape: Option<char>,
    pub progress: GameProgress,
}

impl Default for Game {
    fn default() -> Self {
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

        Self {
            shapes,
            board: Board::new(10, 20),
            state: GameState::NotStarted,
            moving_piece: None,
            next_shape: None,

            progress: GameProgress::new(60, 10),
            moving_piece_timer: Timer::new(),
        }
    }
}

impl Game {
    /// 800 ms (level 0) to 0 ms (max level), reducing faster in the beginning
    fn time_between_moves(&self) -> TimeDelta {
        use std::f32::consts::PI;

        let progress = self.progress.level as f32 / self.progress.levels_to_win as f32;
        let speed_up = (progress * PI / 2.0).sin();
        TimeDelta::milliseconds(((1.0 - speed_up) * 800.0) as i64)
    }

    pub fn on_focus_changed(&mut self, focused: bool) {
        if !focused && self.state == GameState::Running {
            self.state = GameState::Paused;
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
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

    pub fn update(&mut self, time_passed: TimeDelta) {
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

    pub fn piece_collides(&self, piece: Piece) -> bool {
        piece
            .tiles(&self.shapes)
            .iter()
            .any(|&pos| !self.board.contains(pos) || self.board.get_tile(pos).is_some())
    }
}

fn random_shape(values: &[char]) -> char {
    values[rand::rng().random_range(0..values.len())]
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    NotStarted,
    Running,
    Paused,
    GameOver,
}

pub struct Board {
    pub tiles: Vec<Vec<char>>,
    pub width: u8,
    pub height: u8,
}

const EMPTY_TILE: char = 0 as char;

impl Board {
    pub fn get_tile(&self, pos: Pos) -> Option<char> {
        Some(self.tiles[pos.y as usize][pos.x as usize]).filter(|&t| t != EMPTY_TILE)
    }

    pub fn set_tile(&mut self, pos: Pos, tile: char) {
        self.tiles[pos.y as usize][pos.x as usize] = tile;
    }

    pub fn clear_tile(&mut self, pos: Pos) {
        self.tiles[pos.y as usize][pos.x as usize] = EMPTY_TILE;
    }

    pub fn contains(&self, pos: Pos) -> bool {
        let w = self.width;
        let h = self.height;
        let Pos { x, y } = pos;
        if x < 0 || y < 0 {
            return false;
        }
        let (x, y) = (x as u8, y as u8);
        if x >= w || y >= h {
            return false;
        }
        true
    }

    pub fn remove_full_rows(&mut self) -> u8 {
        let mut removed_rows = 0;
        for y in (0..self.height).rev() {
            let mut full_row = true;
            for x in 0..self.width {
                if self.get_tile(Pos::new(x as i8, y as i8)).is_none() {
                    full_row = false;
                    break;
                }
            }
            if full_row {
                removed_rows += 1;
            } else {
                for x in 0..self.width {
                    if let Some(tile) = self.get_tile(Pos::new(x as i8, y as i8)) {
                        self.set_tile(Pos::new(x as i8, (y + removed_rows) as i8), tile);
                    }
                }
            }
            if removed_rows > 0 {
                for x in 0..self.width {
                    self.clear_tile(Pos::new(x as i8, y as i8));
                }
            }
        }
        removed_rows
    }
}

impl Board {
    pub fn new(width: u8, height: u8) -> Board {
        Board {
            tiles: vec![vec![EMPTY_TILE; width as usize]; height as usize],
            width,
            height,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub letter: char,
    rotation: u8,
    origin: Pos,
}

impl Piece {
    pub fn new(letter: char, rotation: u8, origin: Pos) -> Self {
        Self {
            letter,
            rotation,
            origin,
        }
    }

    pub fn moved(mut self, amount: Pos) -> Self {
        self.origin = self.origin + amount;
        self
    }

    pub fn rotated_cw(mut self) -> Self {
        self.rotation = (self.rotation + 1) % 4;
        self
    }

    pub fn tiles<'a, S>(&'a self, shapes: &S) -> [Pos; 4]
    where
        S: Index<&'a char, Output = Shape>,
    {
        shapes[&self.letter].rotated(self.rotation).at(self.origin)
    }
}

#[derive(Clone, Copy)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl Pos {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Shape([Pos; 4]);

impl Shape {
    pub fn new(offsets: [Pos; 4]) -> Self {
        Self(offsets)
    }

    pub fn at(self, pos: Pos) -> [Pos; 4] {
        self.0.map(|off| pos + off)
    }

    pub fn rotated_once(self) -> Self {
        Self(self.0.map(|Pos { x, y }| Pos { x: -y, y: x }))
    }

    pub fn rotated(mut self, times: u8) -> Self {
        for _ in 0..times {
            self = self.rotated_once();
        }
        self
    }
}

pub struct GameProgress {
    pub levels_to_win: u8,
    pub level: u8,

    rows_per_level: u8,
    level_progress: u8,
}

impl GameProgress {
    pub fn new(levels_to_win: u8, rows_per_level: u8) -> Self {
        Self {
            levels_to_win,
            level: 0,

            rows_per_level,
            level_progress: 0,
        }
    }

    pub fn add_rows(&mut self, count: u8) {
        self.level_progress += count;

        while self.level_progress >= self.rows_per_level {
            self.level_progress -= self.rows_per_level;
            self.level += 1;
        }
    }
}
