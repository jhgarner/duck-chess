use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;

use crate::*;

//
// Ideas: There are two boards. There's a lot of overlap between the two, but not all overlap. For
// example, each piece on the different board types are different. But also there's a lot that's
// the same. For example,
//

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

pub trait ChessBoard: Clone + PartialEq + Eq + Default + 'static {
    type Loc: Copy
        + Clone
        + Default
        + std::fmt::Debug
        + Add<Self::Rel, Output = Self::Loc>
        + Serialize
        + DeserializeOwned
        + Hash
        + PartialEq
        + Eq;
    type Rel: Copy + Serialize + DeserializeOwned + Hash + PartialEq + Eq + std::fmt::Debug;
    type Iterator<'a>: Iterator<Item = (Self::Loc, Square)>
    where
        Self: 'a;

    fn get(&self, i: Self::Loc) -> Option<Square>;
    fn valid_locations(
        &self,
        color: Color,
        piece: Piece,
        loc: Self::Loc,
    ) -> HashMap<Self::Loc, ActionRaw<Self::Rel>>;
    fn mk_promotion_board(pieces: Vec<Square>) -> Self;
    fn iter(&self) -> Self::Iterator<'_>;
    fn get_mut(&mut self, i: Self::Loc) -> Option<&mut Square>;
    fn apply(&mut self, loc: Self::Loc, player: Color, action: SingleAction<Self::Rel>);
}

impl ChessBoard for Board {
    type Loc = Loc;
    type Rel = Rel;
    type Iterator<'a> = BoardIter<'a>;

    fn get(&self, i: Self::Loc) -> Option<Square> {
        self.grid.get(i.down)?.get(i.right).copied()
    }

    fn valid_locations(&self, color: Color, piece: Piece, loc: Loc) -> HashMap<Loc, Actions> {
        let board = BoardFocus::new(self, color).focus(loc);
        let mut locations: HashMap<Loc, Actions> = HashMap::new();
        match piece {
            Piece::King { moved } => {
                board.move_with_adj(Piece::King { moved: true }, &mut locations, 1);
                board.move_with_diag(Piece::King { moved: true }, &mut locations, 1);
                board.castle_move(moved, &mut locations);
            }
            Piece::Queen => {
                board.move_with_adj(piece, &mut locations, 8);
                board.move_with_diag(piece, &mut locations, 8);
            }
            Piece::Rook { .. } => {
                board.move_with_adj(Piece::Rook { moved: true }, &mut locations, 8);
            }
            Piece::Bishop => {
                board.move_with_diag(piece, &mut locations, 8);
            }
            Piece::Knight => {
                let knight_moves = [
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ];
                for rel in knight_moves {
                    let rel = Rel::new(rel.0, rel.1);
                    board.add_if_movable(rel, Piece::Knight, &mut locations)
                }
            }
            Piece::Pawn { .. } => {
                board.forward_pawn(&mut locations);
                board.capture_pawn(&mut locations);
                board.en_passant(&mut locations);
                board.mk_promotions(&mut locations);
            }
        }

        locations
    }

    fn mk_promotion_board(pieces: Vec<Square>) -> Self {
        Board { grid: vec![pieces] }
    }

    fn iter(&self) -> BoardIter<'_> {
        BoardIter {
            loc: Loc::default(),
            board: self,
        }
    }

    fn get_mut(&mut self, i: Loc) -> Option<&mut Square> {
        self.grid.get_mut(i.down)?.get_mut(i.right)
    }

    fn apply(&mut self, loc: Loc, player: Color, action: Action) {
        for square in self.squares_mut() {
            square.unpassant_pawns();
        }

        let mut board = BoardFocus::new(self, player).focus(loc);
        match action {
            Action::Move(rel, piece) => {
                board.move_to(rel, piece);
            }
            Action::Castle(side) => {
                board.move_to(side.dir() * 2, Piece::King { moved: true });
                board
                    .shift(side.rook())
                    .move_to(side.rook_to(), Piece::Rook { moved: true });
            }
            Action::EnPassant(target) => {
                board.move_to(target, Piece::Pawn { passantable: false });
                board.remove_at(Rel::new(target.right, 0));
            }
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

pub(crate) struct BoardFocus<RefBoard, Board: ChessBoard> {
    board: RefBoard,
    loc: Board::Loc,
    player: Color,
}

impl<Board: ChessBoard, BoardRef> BoardFocus<BoardRef, Board> {
    pub fn new(board: BoardRef, player: Color) -> Self {
        BoardFocus {
            board,
            player,
            loc: Board::Loc::default(),
        }
    }
}

impl<BoardRef: Deref<Target = Board>, Board: ChessBoard> BoardFocus<BoardRef, Board> {
    pub fn focus(mut self, loc: Board::Loc) -> Self {
        self.loc = loc;
        self
    }

    pub fn shift(&mut self, rel: Board::Rel) -> &mut Self {
        self.loc = self.loc + rel;
        self
    }

    pub fn get(&self, rel: Board::Rel) -> Option<Square> {
        self.board.get(self.loc + rel)
    }

    fn can_move_line(
        &self,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
        line: impl Iterator<Item = Board::Rel>,
    ) {
        for rel in line {
            match self.can_move_to(rel) {
                Some(MoveType::Move(rel)) => {
                    locations.insert(self.loc + rel, ActionRaw::move_it(rel, piece));
                }
                Some(MoveType::Take(rel)) => {
                    locations.insert(self.loc + rel, ActionRaw::move_it(rel, piece));
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn can_move_to(&self, rel: Board::Rel) -> Option<MoveType<Board::Rel>> {
        match self.get(rel) {
            Some(Square::Empty) => Some(MoveType::Move(rel)),
            Some(Square::Piece(to_color, _)) => {
                if self.player != to_color {
                    Some(MoveType::Take(rel))
                } else {
                    None
                }
            }
            Some(Square::Duck) => None,
            None => None,
        }
    }

    pub fn add_if_movable(
        &self,
        rel: Board::Rel,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
    ) {
        if self.can_move_to(rel).is_some() {
            locations.insert(self.loc + rel, ActionRaw::move_it(rel, piece));
        }
    }

    fn add_if_takable(
        &self,
        rel: Board::Rel,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
    ) {
        if let Some(Square::Piece(to_color, _)) = self.get(rel) {
            if self.player != to_color {
                locations.insert(self.loc + rel, ActionRaw::move_it(rel, piece));
            }
        }
    }

    fn add_if_no_take(
        &self,
        rel: Board::Rel,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
    ) {
        if let Some(Square::Empty) = self.get(rel) {
            locations.insert(self.loc + rel, ActionRaw::move_it(rel, piece));
        }
    }
}

impl<Board: ChessBoard> BoardFocus<&mut Board, Board> {
    pub fn remove_at(&mut self, rel: Board::Rel) {
        *self.board.get_mut(self.loc + rel).unwrap() = Square::Empty;
    }

    pub fn move_to(&mut self, rel: Board::Rel, piece: Piece) {
        *self.board.get_mut(self.loc + rel).unwrap() = Square::Piece(self.player, piece);
        *self.board.get_mut(self.loc).unwrap() = Square::Empty;
    }
}

impl BoardFocus<&Board, Board> {
    pub fn move_with_adj(&self, piece: Piece, locations: &mut HashMap<Loc, Actions>, len: i32) {
        let right = Rel::path_to(Rel::new(len, 0));
        let left = Rel::path_to(Rel::new(-len, 0));
        let down = Rel::path_to(Rel::new(0, len));
        let up = Rel::path_to(Rel::new(0, -len));

        self.can_move_line(piece, locations, right);
        self.can_move_line(piece, locations, left);
        self.can_move_line(piece, locations, down);
        self.can_move_line(piece, locations, up);
    }

    pub fn move_with_diag(&self, piece: Piece, locations: &mut HashMap<Loc, Actions>, len: i32) {
        let up_right = Rel::path_to(Rel::new(len, -len));
        let up_left = Rel::path_to(Rel::new(-len, -len));
        let down_right = Rel::path_to(Rel::new(len, len));
        let down_left = Rel::path_to(Rel::new(-len, len));

        self.can_move_line(piece, locations, up_right);
        self.can_move_line(piece, locations, up_left);
        self.can_move_line(piece, locations, down_right);
        self.can_move_line(piece, locations, down_left);
    }

    pub fn castle_move(&self, has_king_moved: bool, locations: &mut HashMap<Loc, Actions>) {
        if !has_king_moved {
            for side in Side::all() {
                if let Some(Square::Piece(_, Piece::Rook { moved: false })) = self.get(side.rook())
                {
                    if Rel::path_to(side.dir() * 2).all(|rel| self.get(rel) == Some(Square::Empty))
                    {
                        locations.insert(self.loc + side.dir() * 2, Actions::castle(side));
                    }
                }
            }
        }
    }

    pub fn forward_pawn(&self, locations: &mut HashMap<Loc, Actions>) {
        let home = match self.player {
            Color::Black => 1,
            Color::White => 6,
        };

        let color = self.player;
        let one_step = Rel::new(0, color.dir());
        self.add_if_no_take(one_step, Piece::Pawn { passantable: false }, locations);
        if self.loc.down == home {
            if let Some(Square::Empty) = self.get(one_step) {
                self.add_if_no_take(
                    Rel::new(0, color.dir() * 2),
                    Piece::Pawn { passantable: true },
                    locations,
                );
            }
        }
    }

    pub fn capture_pawn(&self, locations: &mut HashMap<Loc, Actions>) {
        let color = self.player;
        self.add_if_takable(
            Rel::new(1, color.dir()),
            Piece::Pawn { passantable: false },
            locations,
        );
        self.add_if_takable(
            Rel::new(-1, color.dir()),
            Piece::Pawn { passantable: false },
            locations,
        );
    }

    pub fn en_passant(&self, locations: &mut HashMap<Loc, Actions>) {
        for side in Side::all() {
            if let Some(Square::Piece(other_color, Piece::Pawn { passantable: true })) =
                self.get(side.dir())
            {
                if self.player != other_color {
                    let move_to = side.dir() + Rel::new(0, self.player.dir());
                    locations.insert(self.loc + move_to, Actions::en_passant(move_to));
                }
            }
        }
    }

    pub fn mk_promotions(&self, actions: &mut HashMap<Loc, Actions>) {
        for (loc, action) in std::mem::take(actions).into_iter() {
            if let Actions::Just(Action::Move(rel, _)) = action {
                let new_down = (self.loc + rel).down;
                if new_down == 0 || new_down == 7 {
                    actions.insert(
                        loc,
                        Actions::Promotion(rel, Game::mk_promotion_pieces().into()),
                    );
                } else {
                    actions.insert(loc, action);
                }
            } else {
                actions.insert(loc, action);
            }
        }
    }
}

enum MoveType<Rel> {
    Move(Rel),
    Take(Rel),
}
