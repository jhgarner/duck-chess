use std::{collections::HashMap, ops::Deref};

use crate::{ActionRaw, ChessBoard, Color, Game, Piece, RelIter, SingleAction, Square, SquareId};

pub(crate) struct BoardFocus<RefBoard, Board: ChessBoard> {
    pub board: RefBoard,
    pub loc: Board::Loc,
    pub player: Color,
    pub started_on: Piece,
    pub started_id: SquareId,
}

impl<BoardRef: Deref<Target = Board>, Board: ChessBoard> BoardFocus<BoardRef, Board> {
    pub fn new(board: BoardRef, loc: Board::Loc) -> Option<Self> {
        let (player, started_on, started_id) = board.get(loc).and_then(Square::get_piece)?;
        Some(BoardFocus {
            board,
            player,
            loc,
            started_on,
            started_id,
        })
    }

    pub fn valid_locations_for(&self, turn: Color) -> HashMap<Board::Loc, ActionRaw<Board::Rel>> {
        if turn == self.player {
            self.valid_locations()
        } else {
            HashMap::new()
        }
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
            Some(Square::Piece(to_color, _, _)) => {
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
        if let Some(Square::Piece(to_color, _, _)) = self.get(rel) {
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

    fn valid_locations(&self) -> HashMap<Board::Loc, ActionRaw<Board::Rel>> {
        let mut locations = HashMap::new();
        match self.started_on {
            Piece::King { moved } => {
                self.move_with_adj(Piece::King { moved: true }, &mut locations, 1);
                self.move_with_diag(Piece::King { moved: true }, &mut locations, 1);
                self.castle_move(moved, &mut locations);
            }
            Piece::Queen => {
                self.move_with_adj(Piece::Queen, &mut locations, 8);
                self.move_with_diag(Piece::Queen, &mut locations, 8);
            }
            Piece::Rook { .. } => {
                self.move_with_adj(Piece::Rook { moved: true }, &mut locations, 8);
            }
            Piece::Bishop => {
                self.move_with_diag(Piece::Bishop, &mut locations, 8);
            }
            Piece::Knight => {
                for rel in Board::knight_moves() {
                    self.add_if_movable(rel, Piece::Knight, &mut locations)
                }
            }
            Piece::Pawn { .. } => {
                self.forward_pawn(&mut locations);
                self.capture_pawn(&mut locations);
                self.en_passant(&mut locations);
                self.mk_promotions(&mut locations);
            }
        }

        locations
    }

    pub fn move_with_adj(
        &self,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
        len: u32,
    ) {
        for dir in self.board.rook_dirs() {
            self.can_move_line(piece, locations, RelIter::new(dir, len));
        }
    }
    pub fn castle_move(
        &self,
        has_king_moved: bool,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
    ) {
        if !has_king_moved {
            for castle in self.board.castle_rooks() {
                if let Some(Square::Piece(_, Piece::Rook { moved: false }, _)) =
                    self.get(castle.rook)
                {
                    let mut steps_iter = castle.steps;
                    if steps_iter.all(|rel| self.get(rel) == Some(Square::Empty)) {
                        locations
                            .insert(self.loc + castle.steps.total(), ActionRaw::castle(castle));
                    }
                }
            }
        }
    }

    pub fn move_with_diag(
        &self,
        piece: Piece,
        locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>,
        len: u32,
    ) {
        for dir in self.board.bishop_dirs() {
            self.can_move_line(piece, locations, RelIter::new(dir, len));
        }
    }

    pub fn forward_pawn(&self, locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>) {
        let one_step = Board::forward_one(self.player);
        self.add_if_no_take(one_step, Piece::Pawn { passantable: false }, locations);
        if self.board.home_for(self.player, self.loc) {
            if let Some(Square::Empty) = self.get(one_step) {
                self.add_if_no_take(one_step * 2, Piece::Pawn { passantable: true }, locations);
            }
        }
    }

    pub fn capture_pawn(&self, locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>) {
        for dir in self.board.takeable(self.player) {
            self.add_if_takable(dir, Piece::Pawn { passantable: false }, locations);
        }
    }

    pub fn en_passant(&self, locations: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>) {
        for take_dir in self.board.takeable(self.player) {
            let passed = Board::forward_one(self.player.other());
            if let Some(Square::Piece(other_player, Piece::Pawn { passantable: true }, _)) =
                self.get(passed)
            {
                if self.player != other_player {
                    locations.insert(self.loc + take_dir, ActionRaw::en_passant(take_dir));
                }
            }
        }
    }

    pub fn mk_promotions(&self, actions: &mut HashMap<Board::Loc, ActionRaw<Board::Rel>>) {
        for (loc, action) in std::mem::take(actions).into_iter() {
            if let ActionRaw::Just(SingleAction::Move(rel, _)) = action {
                if self.board.can_promote(self.player, self.loc + rel) {
                    actions.insert(
                        loc,
                        ActionRaw::Promotion(rel, Game::mk_promotion_pieces().into()),
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

impl<Board: ChessBoard> BoardFocus<&mut Board, Board> {
    pub fn remove_at(&mut self, rel: Board::Rel) {
        *self.board.get_mut(self.loc + rel).unwrap() = Square::Empty;
    }

    pub fn move_to(&mut self, rel: Board::Rel, piece: Piece) {
        *self.board.get_mut(self.loc + rel).unwrap() =
            Square::Piece(self.player, piece, self.started_id);
        *self.board.get_mut(self.loc).unwrap() = Square::Empty;
    }

    pub fn apply(&mut self, action: SingleAction<Board::Rel>) {
        for square in self.board.iter_mut() {
            square.unpassant_pawns();
        }
        match action {
            SingleAction::Move(rel, piece) => {
                self.move_to(rel, piece);
            }
            SingleAction::Castle(side) => {
                self.move_to(side.steps.total(), Piece::King { moved: true });
                self.shift(side.rook)
                    .move_to(side.rook_to, Piece::Rook { moved: true });
            }
            SingleAction::EnPassant(target) => {
                self.move_to(target, Piece::Pawn { passantable: false });
                let passed = Board::forward_one(self.player.other());
                self.remove_at(target + passed);
            }
        }
    }
}

enum MoveType<Rel> {
    Move(Rel),
    Take(Rel),
}
