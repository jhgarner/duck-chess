use std::{num::TryFromIntError, ops::Add};

use serde::{Deserialize, Serialize};

use super::dir::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default,
)]
pub struct Coord {
    pub q: i32,
    pub r: i32,
}

impl Coord {
    pub fn center() -> Coord {
        Coord { q: 0, r: 0 }
    }

    pub fn new(q: i32, r: i32) -> Coord {
        Coord { q, r }
    }

    pub fn dist(&self) -> f64 {
        let s = 0 - self.q - self.r;
        f64::sqrt((s.pow(2) + self.q.pow(2) + self.r.pow(2)).into())
    }

    pub fn to_xy(self, radius: u32) -> Result<(usize, usize), TryFromIntError> {
        let radius = radius as i32;
        let y = (self.r + radius).try_into()?;
        let x = (self.q + 0.min(self.r) + radius).try_into()?;
        Ok((x, y))
    }

    pub fn from_xy(x: usize, y: usize, radius: u32) -> Coord {
        let r = y as i32 - radius as i32;
        let q = x as i32 - 0.min(r) - radius as i32;
        Coord { r, q }
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
