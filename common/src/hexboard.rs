mod coord;

pub use coord::*;
mod dir;
pub use dir::*;
mod iter;
use iter::*;

use crate::{ChessBoard, Color, RelIter, Square};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hexboard {
    radius: u32,
    grid: Vec<Vec<Square>>,
}

impl Default for Hexboard {
    fn default() -> Self {
        Self::new(5)
    }
}

impl Hexboard {
    pub fn new(radius: u32) -> Hexboard {
        let mut grid = Vec::new();
        for i in 0..=radius * 2 {
            let r = i as i32 - radius as i32;
            let inner_size = 2 * radius as i32 + 1 - (radius as i32 - r).abs();
            grid.push(vec![Square::Empty; inner_size as usize]);
        }
        Hexboard { radius, grid }
    }

    fn pawn_squares(&self, color: Color) -> Vec<Coord> {
        let start = Coord {
            q: 0,
            r: -color.dir(),
        };
        let mut locs: Vec<Coord> = vec![start];

        for rel in RelIter::new(Dir::new(-color.dir(), 0), self.radius) {
            locs.push(start + rel);
        }
        for rel in RelIter::new(Dir::new(color.dir(), -color.dir()), self.radius) {
            locs.push(start + rel);
        }

        locs
    }

    fn promote_squares(&self, color: Color) -> Vec<Coord> {
        let start = Coord {
            q: 0,
            r: color.dir() * self.radius as i32,
        };
        let mut locs: Vec<Coord> = vec![start];

        for rel in RelIter::new(Dir::new(color.dir(), 0), self.radius) {
            locs.push(start + rel);
        }
        for rel in RelIter::new(Dir::new(-color.dir(), color.dir()), self.radius) {
            locs.push(start + rel);
        }

        locs
    }
}

impl ChessBoard for Hexboard {
    type Loc = Coord;
    type Rel = Dir;
    fn get(&self, coord: Coord) -> Option<Square> {
        let (x, y) = coord.to_xy(self.radius).ok()?;
        let inner = self.grid.get(x)?;
        inner.get(y).copied()
    }

    fn get_mut(&mut self, coord: Coord) -> Option<&mut Square> {
        let inner = self.grid.get_mut(coord.q as usize)?;
        inner.get_mut(0.max(self.radius as i32 - coord.r) as usize)
    }

    fn iter(&self) -> impl Iterator<Item = (Coord, Square)> {
        HexIter {
            x: 0,
            y: 0,
            board: self,
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Square> {
        self.grid.iter_mut().flat_map(|row| row.iter_mut())
    }

    fn knight_moves() -> impl IntoIterator<Item = Self::Rel> {
        [
            Dir::new(-2, -1),
            Dir::new(-1, -2),
            Dir::new(1, -3),
            Dir::new(2, -3),
            Dir::new(3, -2),
            Dir::new(3, -1),
            Dir::new(2, 1),
            Dir::new(1, 2),
            Dir::new(-1, 3),
            Dir::new(-2, 3),
            Dir::new(-3, 2),
            Dir::new(-3, 1),
        ]
    }

    fn forward_one(color: Color) -> Self::Rel {
        Dir::new(0, color.dir())
    }

    fn home_for(&self, color: Color, loc: Self::Loc) -> bool {
        self.pawn_squares(color).contains(&loc)
    }

    fn takeable(&self, color: Color) -> impl IntoIterator<Item = Self::Rel> {
        [
            Dir::new(color.dir(), 0),
            Dir::new(-color.dir(), color.dir()),
        ]
    }

    fn can_promote(&self, color: Color, loc: Self::Loc) -> bool {
        self.promote_squares(color).contains(&loc)
    }

    fn rook_dirs(&self) -> impl IntoIterator<Item = Self::Rel> {
        [
            Dir::new(0, 1),
            Dir::new(0, -1),
            Dir::new(1, 0),
            Dir::new(-1, 0),
            Dir::new(1, -1),
            Dir::new(-1, 1),
        ]
    }

    fn castle_rooks(&self) -> impl IntoIterator<Item = crate::Castle<Self::Rel>> {
        []
    }

    fn bishop_dirs(&self) -> impl IntoIterator<Item = Self::Rel> {
        [
            Dir::new(-1, -1),
            Dir::new(1, -2),
            Dir::new(2, -1),
            Dir::new(1, 1),
            Dir::new(-1, 2),
            Dir::new(-2, 1),
        ]
    }
}
