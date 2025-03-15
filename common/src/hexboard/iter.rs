use crate::ChessBoard;
use crate::Square;

use super::{Coord, Dir, Hexboard};

pub struct HexIter<'r> {
    pub board: &'r Hexboard,
    pub x: usize,
    pub y: usize,
}

impl Iterator for HexIter<'_> {
    type Item = (Coord, Square);

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.board.grid.get(self.y)?;
        row.get(self.x)
            .map(|col| (Coord::from_xy(self.x, self.y, self.board.radius), *col))
            .or_else(|| {
                self.x = 0;
                self.y += 1;
                self.next()
            })
    }
}

pub struct DirHexIter<'r> {
    pub board: &'r Hexboard,
    pub at: Coord,
    pub dir: Dir,
}

impl Iterator for DirHexIter<'_> {
    type Item = (Coord, Square);

    fn next(&mut self) -> Option<Self::Item> {
        self.at = self.at + self.dir;
        Some((self.at, self.board.get(self.at)?))
    }
}
