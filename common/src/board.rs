use menuboard::MenuBoard;
use once_cell::sync::Lazy;

use crate::*;

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    pub grid: Vec<Vec<Square>>,
}

pub struct BoardIter<'a> {
    loc: Loc,
    board: &'a Board,
}

impl Iterator for BoardIter<'_> {
    type Item = (Loc, Square);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.board.get(self.loc) {
            let at = self.loc;
            self.loc.right += 1;
            Some((at, next))
        } else {
            self.loc.right = 0;
            self.loc.down += 1;
            let at = self.loc;
            self.loc.right += 1;
            self.board.get(at).map(|square| (at, square))
        }
    }
}

impl ChessBoard for Board {
    type Loc = Loc;
    type Rel = Rel;
    // type Iterator<'a> = BoardIter<'a>;
    // type MutIterator<'a> = BoardIter<'a>;

    fn get(&self, i: Self::Loc) -> Option<Square> {
        self.grid.get(i.down)?.get(i.right).copied()
    }

    fn knight_moves() -> impl IntoIterator<Item = Self::Rel> {
        [
            Rel::new(2, 1),
            Rel::new(2, -1),
            Rel::new(-2, 1),
            Rel::new(-2, -1),
            Rel::new(1, 2),
            Rel::new(1, -2),
            Rel::new(-1, 2),
            Rel::new(-1, -2),
        ]
    }

    fn iter(&self) -> impl Iterator<Item = (Loc, Square)> {
        BoardIter {
            loc: Loc::default(),
            board: self,
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Square> {
        self.grid.iter_mut().flat_map(|row| row.iter_mut())
    }

    fn get_mut(&mut self, i: Loc) -> Option<&mut Square> {
        self.grid.get_mut(i.down)?.get_mut(i.right)
    }

    fn forward_one(color: Color) -> Self::Rel {
        Rel::new(0, color.dir())
    }

    fn home_for(&self, color: Color, loc: Self::Loc) -> bool {
        let home = match color {
            Color::Black => 1,
            Color::White => 6,
        };
        loc.down == home
    }

    fn takeable(&self, color: Color) -> impl IntoIterator<Item = Self::Rel> {
        [Rel::new(1, color.dir()), Rel::new(-1, color.dir())]
    }

    fn can_promote(&self, color: Color, loc: Self::Loc) -> bool {
        let goal = match color {
            Color::Black => 7,
            Color::White => 0,
        };
        loc.down == goal
    }

    fn rook_dirs(&self) -> impl IntoIterator<Item = Self::Rel> {
        [
            Rel::new(1, 0),
            Rel::new(-1, 0),
            Rel::new(0, 1),
            Rel::new(0, -1),
        ]
    }

    fn castle_rooks(&self) -> impl IntoIterator<Item = Castle<Self::Rel>> {
        [
            Castle {
                rook: Rel::new(3, 0),
                rook_to: Rel::new(-2, 0),
                steps: RelIter::new(Rel::new(1, 0), 2),
            },
            Castle {
                rook: Rel::new(-4, 0),
                rook_to: Rel::new(3, 0),
                steps: RelIter::new(Rel::new(-1, 0), 2),
            },
        ]
    }
    fn bishop_dirs(&self) -> impl IntoIterator<Item = Self::Rel> {
        [
            Rel::new(1, 1),
            Rel::new(-1, 1),
            Rel::new(1, -1),
            Rel::new(-1, -1),
        ]
    }
}

impl From<MenuBoard> for Board {
    fn from(value: MenuBoard) -> Self {
        Board {
            grid: vec![value.grid],
        }
    }
}

impl Board {
    pub fn static_default() -> &'static Board {
        static DEFAULT_BOARD: Lazy<Board> = Lazy::new(Board::default);

        &DEFAULT_BOARD
    }

    pub fn squares(&self) -> impl Iterator<Item = &Square> {
        self.grid.iter().flat_map(|row| row.iter())
    }

    pub fn squares_mut(&mut self) -> impl Iterator<Item = &mut Square> {
        self.grid.iter_mut().flat_map(|row| row.iter_mut())
    }

    pub fn rows(&self) -> impl Iterator<Item = &Vec<Square>> {
        self.grid.iter()
    }

    pub fn width(&self) -> usize {
        self.grid.first().map_or(0, |row| row.len())
    }

    pub fn height(&self) -> usize {
        self.grid.len()
    }

    pub fn empties(&self) -> impl Iterator<Item = Loc> + '_ {
        // Is this better or worse than building up an intermediate Vec using for loops?
        self.grid.iter().enumerate().flat_map(|(down, row)| {
            row.iter().enumerate().filter_map(move |(right, square)| {
                if let Square::Empty = square {
                    Some(Loc::new(right, down))
                } else {
                    None
                }
            })
        })
    }

    pub fn duck(&self) -> Option<Loc> {
        // Is this better or worse than find_map(...find_map)?
        for (down, row) in self.grid.iter().enumerate() {
            for (right, square) in row.iter().enumerate() {
                if let Square::Duck = square {
                    return Some(Loc::new(right, down));
                }
            }
        }
        None
    }
}

impl Default for Board {
    fn default() -> Self {
        use Piece::*;

        let rook = Rook { moved: false };
        let king = King { moved: false };
        let back = [rook, Knight, Bishop, Queen, king, Bishop, Knight, rook];
        let front = [Pawn { passantable: false }; 8];
        let empty = vec![Square::Empty; 8];
        let board = vec![
            back.iter()
                .map(|piece| Square::Piece(Color::Black, *piece))
                .collect(),
            front
                .iter()
                .map(|piece| Square::Piece(Color::Black, *piece))
                .collect(),
            empty.clone(),
            empty.clone(),
            empty.clone(),
            empty,
            front
                .iter()
                .map(|piece| Square::Piece(Color::White, *piece))
                .collect(),
            back.iter()
                .map(|piece| Square::Piece(Color::White, *piece))
                .collect(),
        ];
        Board { grid: board }
    }
}
