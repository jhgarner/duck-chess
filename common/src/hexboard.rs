mod coord;

pub use coord::*;
mod dir;
pub use dir::*;
mod iter;
use iter::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{ChessBoard, Color, Piece, RelIter, SomeTurn, Square, TurnRaw};

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Hexboard {
    pub grid: Vec<Vec<Square>>,
}

impl Default for Hexboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Hexboard {
    pub fn new() -> Hexboard {
        let mut grid = Vec::new();
        let rad: i32 = 5;
        for i in 0..=rad * 2 {
            let inner_size = 2 * rad + 1 - (rad - i).abs();
            grid.push(vec![Square::Empty; inner_size as usize]);
        }
        let mut board = Hexboard { grid };

        // Set all the initial pieces
        for color in Color::all() {
            for pawn in Self::pawn_squares(color) {
                *board.get_mut(pawn).unwrap() =
                    Square::piece(color, Piece::Pawn { passantable: false });
            }
            for bishop in 3..=5 {
                let coord = Coord::new(0, bishop * -color.dir());
                *board.get_mut(coord).unwrap() = Square::piece(color, Piece::Bishop);
            }
        }
        *board.get_mut(Coord::new(1, 4)).unwrap() =
            Square::piece(Color::White, Piece::King { moved: false });
        *board.get_mut(Coord::new(1, -5)).unwrap() =
            Square::piece(Color::Black, Piece::King { moved: false });
        *board.get_mut(Coord::new(-1, 5)).unwrap() = Square::piece(Color::White, Piece::Queen);
        *board.get_mut(Coord::new(-1, -4)).unwrap() = Square::piece(Color::Black, Piece::Queen);
        *board.get_mut(Coord::new(2, 3)).unwrap() = Square::piece(Color::White, Piece::Knight);
        *board.get_mut(Coord::new(-2, 5)).unwrap() = Square::piece(Color::White, Piece::Knight);
        *board.get_mut(Coord::new(2, -5)).unwrap() = Square::piece(Color::Black, Piece::Knight);
        *board.get_mut(Coord::new(-2, -3)).unwrap() = Square::piece(Color::Black, Piece::Knight);
        *board.get_mut(Coord::new(3, 2)).unwrap() =
            Square::piece(Color::White, Piece::Rook { moved: false });
        *board.get_mut(Coord::new(-3, 5)).unwrap() =
            Square::piece(Color::White, Piece::Rook { moved: false });
        *board.get_mut(Coord::new(3, -5)).unwrap() =
            Square::piece(Color::Black, Piece::Rook { moved: false });
        *board.get_mut(Coord::new(-3, -2)).unwrap() =
            Square::piece(Color::Black, Piece::Rook { moved: false });
        board
    }

    pub fn static_default() -> &'static Hexboard {
        static DEFAULT_BOARD: Lazy<Hexboard> = Lazy::new(Hexboard::default);

        &DEFAULT_BOARD
    }

    fn pawn_squares(color: Color) -> Vec<Coord> {
        let start = Coord {
            q: 0,
            r: -color.dir(),
        };
        let mut locs: Vec<Coord> = vec![start];

        for rel in RelIter::new(Dir::new(-color.dir(), 0), 4) {
            locs.push(start + rel);
        }
        for rel in RelIter::new(Dir::new(color.dir(), -color.dir()), 4) {
            locs.push(start + rel);
        }

        locs
    }

    fn promote_squares(color: Color) -> Vec<Coord> {
        let start = Coord {
            q: 0,
            r: color.dir() * 5,
        };
        let mut locs: Vec<Coord> = vec![start];

        for rel in RelIter::new(Dir::new(color.dir(), 0), 5) {
            locs.push(start + rel);
        }
        for rel in RelIter::new(Dir::new(-color.dir(), color.dir()), 5) {
            locs.push(start + rel);
        }

        locs
    }
}

impl ChessBoard for Hexboard {
    type Loc = Coord;
    type Rel = Dir;
    fn get(&self, coord: Coord) -> Option<Square> {
        let (x, y) = coord.to_xy(5).ok()?;
        let inner = self.grid.get(y)?;
        inner.get(x).copied()
    }

    fn get_mut(&mut self, coord: Coord) -> Option<&mut Square> {
        let (x, y) = coord.to_xy(5).ok()?;
        let inner = self.grid.get_mut(y)?;
        inner.get_mut(x)
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
        Self::pawn_squares(color).contains(&loc)
    }

    fn takeable(&self, color: Color) -> impl IntoIterator<Item = Self::Rel> {
        [
            Dir::new(color.dir(), 0),
            Dir::new(-color.dir(), color.dir()),
        ]
    }

    fn can_promote(&self, color: Color, loc: Self::Loc) -> bool {
        Self::promote_squares(color).contains(&loc)
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

    fn wrap_turn(turn: TurnRaw<Self>) -> SomeTurn {
        SomeTurn::Hex(turn)
    }
}
