use std::ops::{Add, Mul};

pub use DirHoriz::*;
pub use DirVert::*;

pub static ZERO_DIR: Dir = Dir { q: 0, r: 0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dir {
    pub q: i16,
    pub r: i16,
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

impl Mul<i16> for Dir {
    type Output = Dir;

    fn mul(self, rhs: i16) -> Self::Output {
        Dir {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}

pub enum DirVert {
    Up,
    Down,
}

impl From<DirVert> for Dir {
    fn from(value: DirVert) -> Self {
        match value {
            DirVert::Up => Dir { q: 0, r: -1 },
            DirVert::Down => Dir { q: 0, r: 1 },
        }
    }
}

impl Mul<i16> for DirVert {
    type Output = Dir;

    fn mul(self, rhs: i16) -> Self::Output {
        let lhs: Dir = self.into();
        Dir {
            q: lhs.q * rhs,
            r: lhs.r * rhs,
        }
    }
}

impl<T: Into<Dir>> Add<T> for DirVert {
    type Output = Dir;

    fn add(self, rhs: T) -> Self::Output {
        let lhs: Dir = self.into();
        let rhs: Dir = rhs.into();
        Dir {
            q: lhs.q + rhs.q,
            r: lhs.r + rhs.r,
        }
    }
}

impl DirVert {
    fn to_r(&self) -> i16 {
        match self {
            DirVert::Up => -1,
            DirVert::Down => 0,
        }
    }

    pub fn and(&self, horiz: DirHoriz) -> Dir {
        Dir {
            q: horiz.to_q(),
            r: self.to_r(),
        }
    }
}

pub enum DirHoriz {
    Right,
    Left,
}

impl DirHoriz {
    fn to_q(&self) -> i16 {
        match self {
            DirHoriz::Right => 1,
            DirHoriz::Left => -1,
        }
    }
}
