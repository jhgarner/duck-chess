use std::ops::Add;

use super::dir::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord {
    pub q: i16,
    pub r: i16,
}

impl Coord {
    pub fn center() -> Coord {
        Coord { q: 0, r: 0 }
    }

    pub fn dist(&self) -> f32 {
        let s = 0 - self.q - self.r;
        f32::sqrt((s.pow(2) + self.q.pow(2) + self.r.pow(2)).into())
    }
}

impl<T: Into<Dir>> Add<T> for Coord {
    type Output = Coord;

    fn add(self, rhs: T) -> Self::Output {
        let rhs: Dir = rhs.into();
        Coord {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
