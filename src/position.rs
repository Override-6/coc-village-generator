use crate::cell::Cell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Eq for Pos {}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_cell(&self) -> Cell {
        Cell {
            x: self.x as i16,
            y: self.y as i16,
        }
    }

    pub fn distance(&self, other: Pos) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Hash for Pos {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.x.to_bits());
        state.write_u32(self.y.to_bits());
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x < other.x {
            Ordering::Less
        } else if self.x > other.x {
            Ordering::Greater
        } else if self.y < other.y {
            Ordering::Less
        } else if self.y > other.y {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
