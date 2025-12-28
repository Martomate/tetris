use std::ops::{Add, Index};

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
