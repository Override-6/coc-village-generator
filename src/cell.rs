use crate::position::Pos;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct Cell {
    pub x: i16,
    pub y: i16,
}

impl Cell {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn to_pos(&self) -> Pos {
        Pos {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub fn distance(&self, other: Cell) -> f32 {
        (((self.x - other.x) as f32).powi(2) + ((self.y - other.y) as f32).powi(2)).sqrt()
    }

    pub fn distance_with_pos(&self, other: Pos) -> f32 {
        ((self.x as f32 - other.x).powi(2) + (self.y as f32 - other.y).powi(2)).sqrt()
    }
}
