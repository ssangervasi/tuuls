use std::collections::HashMap;

use crate::position::{Position, Visible};

const BLANK: char = ' ';

#[derive(Clone, Debug)]
pub struct Screen {
    pub cols: i32,
    pub rows: i32,
    hash_map: HashMap<Position, char>,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            cols: 100,
            rows: 100,
            hash_map: HashMap::with_capacity(10 * 10),
        }
    }
}

impl Screen {
    pub fn with_size(col_row: Position) -> Self {
        Self {
            cols: col_row.col,
            rows: col_row.row,
            hash_map: HashMap::with_capacity(10 * 10),
        }
    }

    pub fn write(&mut self, position: &Position, ch: char) -> char {
        match ch {
            BLANK => self.hash_map.remove(position),
            _ => self.hash_map.insert(*position, ch),
        }
        .unwrap_or(BLANK)
    }

    pub fn read(&self, position: &Position) -> char {
        match self.hash_map.get(position) {
            Some(&ch) => ch,
            None => BLANK,
        }
    }

    pub(crate) fn mem(&self) -> usize {
        (std::mem::size_of::<Position>() + std::mem::size_of::<char>()) * self.hash_map.len()
    }

    pub fn clip(&self, position: &Position) -> Visible {
        (*position).into()
    }
}
