use std::ops::{Deref, DerefMut};

use crate::{
    Action, Color, Game, Loc,
    Piece::{self, *},
    Rel, Side,
    Square::{self, *},
};

struct BoardFocus<T> {
    game: T,
    loc: Loc,
}

type BoardFocusImut<'a> = &'a BoardFocus<&'a Game>;

impl<T: Deref<Target = Game>> BoardFocus<T> {
    pub fn new(game: T) -> Self {
        BoardFocus {
            game,
            loc: Loc::new(0, 0),
        }
    }

    pub fn focus(&mut self, loc: Loc) -> &mut Self {
        self.loc = loc;
        self
    }

    pub fn shift(&mut self, rel: Rel) -> &mut Self {
        self.loc = rel.from(self.loc);
        self
    }

    pub fn get(&self, rel: Rel) -> Option<Square> {
        self.game.get(rel.from(self.loc))
    }

    pub fn focused(&self) -> Square {
        self.game.get(self.loc).unwrap()
    }
}

impl<T: DerefMut<Target = Game>> BoardFocus<T> {
    pub fn remove_at(&mut self, rel: Rel) {
        *self.game.get_mut(rel.from(self.loc)) = Square::Empty;
    }

    pub fn move_to(&mut self, rel: Rel) {
        let piece = self.focused();
        *self.game.get_mut(rel.from(self.loc)) = piece.moves(rel);
        *self.game.get_mut(self.loc) = Square::Empty;
    }

    pub fn set(&mut self, square: Square) {
        *self.game.get_mut(self.loc) = square;
    }
}

pub fn apply(game: &mut Game, loc: Loc, action: Action) {
    for square in game.squares_mut() {
        square.unpassant_pawns();
    }

    let mut board = BoardFocus::new(game);
    board.focus(loc);
    match action {
        Action::Move(rel) => {
            board.move_to(rel);
        }
        Action::Castle(side) => {
            board.move_to(side.king_to());
            board.shift(side.rook()).move_to(side.dir() * 2);
        }
        Action::EnPassant(side) => {
            let target = side.dir() + Rel::new(0, board.game.turn().dir());
            board.move_to(target);
            board.remove_at(side.dir());
        }
        Action::Promote(rel, to) => {
            board.set(to);
            board.move_to(rel);
        }
    }
}

pub fn valid_locations(game: &Game, loc: Loc) -> Vec<Action> {
    let mut board = BoardFocus::new(game);
    let mut locations: Vec<Action> = Vec::new();
    if let Square::Piece(color, piece) = board.focus(loc).focused() {
        if color != game.turn() {
            return Vec::new();
        }
        match piece {
            King { moved } => {
                move_with_adj(&board, &mut locations, 1);
                move_with_diag(&board, &mut locations, 1);
                castle_move(&board, moved, &mut locations);
            }
            Queen => {
                move_with_adj(&board, &mut locations, 8);
                move_with_diag(&board, &mut locations, 8);
            }
            Rook { .. } => {
                move_with_adj(&board, &mut locations, 8);
            }
            Bishop => {
                move_with_diag(&board, &mut locations, 8);
            }
            Knight => {
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
                    add_if_movable(&board, rel, &mut locations)
                }
            }
            Pawn { .. } => {
                forward_pawn(&board, loc, &mut locations);
                capture_pawn(&board, &mut locations);
                en_passant(&board, &mut locations);
                mk_promotions(&board, loc, &mut locations);
            }
        }
    }

    locations
}

fn move_with_adj(board: BoardFocusImut, locations: &mut Vec<Action>, len: i32) {
    let right = Rel::path_to(Rel::new(len, 0));
    let left = Rel::path_to(Rel::new(-len, 0));
    let down = Rel::path_to(Rel::new(0, len));
    let up = Rel::path_to(Rel::new(0, -len));

    can_move_line(board, locations, right);
    can_move_line(board, locations, left);
    can_move_line(board, locations, down);
    can_move_line(board, locations, up);
}

fn move_with_diag(board: BoardFocusImut, locations: &mut Vec<Action>, len: i32) {
    let up_right = Rel::path_to(Rel::new(len, -len));
    let up_left = Rel::path_to(Rel::new(-len, -len));
    let down_right = Rel::path_to(Rel::new(len, len));
    let down_left = Rel::path_to(Rel::new(-len, len));

    can_move_line(board, locations, up_right);
    can_move_line(board, locations, up_left);
    can_move_line(board, locations, down_right);
    can_move_line(board, locations, down_left);
}

enum MoveType {
    Move(Rel),
    Take(Rel),
}

impl MoveType {
    pub fn get(&self) -> Rel {
        match self {
            MoveType::Move(rel) => *rel,
            MoveType::Take(rel) => *rel,
        }
    }
}

fn can_move_line(
    board: BoardFocusImut,
    locations: &mut Vec<Action>,
    line: impl Iterator<Item = Rel>,
) {
    for rel in line {
        match can_move_to(board, rel) {
            Some(MoveType::Move(rel)) => {
                locations.push(Action::Move(rel));
            }
            Some(MoveType::Take(rel)) => {
                locations.push(Action::Move(rel));
                break;
            }
            _ => {
                break;
            }
        }
    }
}

fn can_move_to(board: BoardFocusImut, rel: Rel) -> Option<MoveType> {
    match board.get(rel) {
        Some(Empty) => Some(MoveType::Move(rel)),
        Some(Piece(to_color, _)) => {
            if board.game.turn() != to_color {
                Some(MoveType::Take(rel))
            } else {
                None
            }
        }
        Some(Duck) => None,
        None => None,
    }
}

fn add_if_movable(board: BoardFocusImut, rel: Rel, locations: &mut Vec<Action>) {
    can_move_to(board, rel).map(|move_type| locations.push(Action::Move(move_type.get())));
}

fn castle_move(board: BoardFocusImut, king_moved: bool, locations: &mut Vec<Action>) {
    if !king_moved {
        for side in Side::all() {
            if let Some(Piece(_, Rook { moved: false })) = board.get(side.rook()) {
                if Rel::path_to(side.king_to()).all(|rel| board.get(rel) == Some(Square::Empty)) {
                    locations.push(Action::Castle(side));
                }
            }
        }
    }
}

fn forward_pawn(board: BoardFocusImut, loc: Loc, locations: &mut Vec<Action>) {
    let home = match board.game.turn() {
        Color::Black => 1,
        Color::White => 6,
    };

    let color = board.game.turn();
    let one_step = Rel::new(0, color.dir());
    add_if_no_take(&board, one_step, locations);
    if loc.down == home {
        if let Some(Square::Empty) = board.get(one_step) {
            add_if_no_take(&board, Rel::new(0, color.dir() * 2), locations);
        }
    }
}

fn capture_pawn(board: BoardFocusImut, locations: &mut Vec<Action>) {
    let color = board.game.turn();
    add_if_takable(board, Rel::new(1, color.dir()), locations);
    add_if_takable(board, Rel::new(-1, color.dir()), locations);
}

fn add_if_takable(board: BoardFocusImut, rel: Rel, locations: &mut Vec<Action>) {
    if let Some(Piece(to_color, _)) = board.get(rel) {
        if board.game.turn() != to_color {
            locations.push(Action::Move(rel))
        }
    }
}

fn add_if_no_take(board: BoardFocusImut, rel: Rel, locations: &mut Vec<Action>) {
    if let Some(Square::Empty) = board.get(rel) {
        locations.push(Action::Move(rel))
    }
}

fn en_passant(board: BoardFocusImut, locations: &mut Vec<Action>) {
    for side in Side::all() {
        if let Some(Piece(other_color, Piece::Pawn { passantable })) = board.get(side.dir()) {
            if board.game.turn() != other_color && passantable {
                locations.push(Action::EnPassant(side));
            }
        }
    }
}

pub fn valid_duck(game: &Game, loc: Loc) -> bool {
    game.get(loc) == Some(Square::Empty)
}

pub fn apply_duck(game: &mut Game, loc: Loc) {
    for square in game.squares_mut() {
        if let Square::Duck = square {
            *square = Square::Empty;
        }
    }
    *game.get_mut(loc) = Square::Duck;
}

pub fn game_over(game: &Game) -> Option<Color> {
    if !game
        .squares()
        .any(|square| matches!(square, Square::Piece(Color::White, Piece::King { .. })))
    {
        Some(Color::Black)
    } else if !game
        .squares()
        .any(|square| matches!(square, Square::Piece(Color::Black, Piece::King { .. })))
    {
        Some(Color::White)
    } else {
        None
    }
}

pub fn mk_board(game: &Game) -> [[Square; 4]; 1] {
    let color = game.turn();
    [[
        Square::Piece(color, Piece::Queen),
        Square::Piece(color, Piece::Knight),
        Square::Piece(color, Piece::Rook { moved: true }),
        Square::Piece(color, Piece::Bishop),
    ]]
}

fn mk_promotions(board: BoardFocusImut, loc: Loc, actions: &mut Vec<Action>) {
    let mut new_actions = Vec::new();
    for action in actions.iter() {
        if let Action::Move(rel) = action {
            let new_down = rel.from(loc).down;
            if new_down == 0 || new_down == 7 {
                for square in mk_board(board.game)[0] {
                    let action = Action::Promote(*rel, square);
                    new_actions.push(action);
                }
            } else {
                new_actions.push(*action);
            }
        } else {
            new_actions.push(*action);
        }
    }
    *actions = new_actions;
}
