use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cell {
    pub x: i16,
    pub y: i16,
}

impl Cell {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y < other.y {
            Some(Ordering::Less)
        } else if self.y > other.y {
            Some(Ordering::Greater)
        } else if self.x < other.x {
            Some(Ordering::Less)
        } else if self.x > other.x {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}