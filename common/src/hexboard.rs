mod coord;
pub use coord::*;
mod dir;
pub use dir::*;
mod iter;
use iter::*;

pub struct Hexboard<T> {
    radius: i16,
    grid: Vec<Vec<T>>,
}

impl<T: Default + Copy> Hexboard<T> {
    pub fn new(radius: i16) -> Hexboard<T> {
        let mut grid = Vec::new();
        for i in -radius..=radius {
            let inner_size = 2 * radius + 1 - (radius - i).abs();
            grid.push(vec![T::default(); inner_size as usize]);
        }
        Hexboard { radius, grid }
    }
}

impl<T> Hexboard<T> {
    pub fn get(&self, coord: Coord) -> Option<&T> {
        let inner = self.grid.get(coord.q as usize)?;
        inner.get(0.max(self.radius - coord.r) as usize)
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut T> {
        let inner = self.grid.get_mut(coord.q as usize)?;
        inner.get_mut(0.max(self.radius - coord.r) as usize)
    }

    pub fn iter(&self, origin: Coord, dir: Dir) -> impl Iterator<Item = (Coord, &T)> {
        HexIter {
            board: self,
            at: origin,
            dir,
        }
    }

    pub fn iter_mut(&mut self, origin: Coord, dir: Dir) -> impl Iterator<Item = (Coord, &mut T)> {
        HexIterMut {
            board: self,
            at: origin,
            dir,
        }
    }
}
