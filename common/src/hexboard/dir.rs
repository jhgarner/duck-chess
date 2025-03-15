use std::ops::{Add, Mul};

use serde::{Deserialize, Serialize};

pub static ZERO_DIR: Dir = Dir { q: 0, r: 0 };

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default,
)]
pub struct Dir {
    pub q: i32,
    pub r: i32,
}

impl Dir {
    pub fn new(q: i32, r: i32) -> Dir {
        Dir { q, r }
    }
}

impl<T: Into<Dir>> Add<T> for Dir {
    type Output = Dir;

    fn add(self, rhs: T) -> Self::Output {
        let rhs: Dir = rhs.into();
        Dir {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl Mul<i32> for Dir {
    type Output = Dir;

    fn mul(self, rhs: i32) -> Self::Output {
        Dir {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}
