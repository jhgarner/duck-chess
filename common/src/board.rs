use once_cell::sync::Lazy;

use crate::*;

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    // You might be thinking, "Why use a Vec when [_; 8] represents a chessboard?" Well, we also
    // use a Board to represent the 4x1 board that's displayed when promoting a pawn. That was easy
    // to code like that because I'd already implemented board rendering and click detection. It
    // might make sense to switch the UI for promotion to something else instead of relying on a
    // weirdly sized chess board. Even then, I think it would be really cool if we could define
    // other chess variants to run in this program. There's at least one chess variant which uses
    // two boards and representing that as one 16x8 board might be nice, so I'm tempted to leave
    // this generic.
    // You might be wondering, "Why use a Vec when const generic arrays exist?" Well, that makes
    // some of the frontend code which tries to store the board a little more complicated (requires
    // pulling in PhantomData and other fun stuff). I'd be willing to put up with that, but there's
    // a block of code which wants to return a board of unknown size. Down one code path it wants
    // to return a 4x1 board and down the rest of the paths it wants to return an 8x8 board. The
    // resulting board is passed to a function which accepts boards of any size. Instead of having
    // an existential crisis and trying to use dyn traits, I just gave up and added more heap
    // allocations.
    pub grid: Vec<Vec<Square>>,
}

impl Board {
    pub fn static_default() -> &'static Board {
        static DEFAULT_BOARD: Lazy<Board> = Lazy::new(|| Board::default());

        &*DEFAULT_BOARD
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

pub(crate) struct BoardFocus<T> {
    pub game: T,
    loc: Loc,
}

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

    pub fn move_with_adj(&self, locations: &mut Vec<Action>, len: i32) {
        let right = Rel::path_to(Rel::new(len, 0));
        let left = Rel::path_to(Rel::new(-len, 0));
        let down = Rel::path_to(Rel::new(0, len));
        let up = Rel::path_to(Rel::new(0, -len));

        self.can_move_line(locations, right);
        self.can_move_line(locations, left);
        self.can_move_line(locations, down);
        self.can_move_line(locations, up);
    }

    pub fn move_with_diag(&self, locations: &mut Vec<Action>, len: i32) {
        let up_right = Rel::path_to(Rel::new(len, -len));
        let up_left = Rel::path_to(Rel::new(-len, -len));
        let down_right = Rel::path_to(Rel::new(len, len));
        let down_left = Rel::path_to(Rel::new(-len, len));

        self.can_move_line(locations, up_right);
        self.can_move_line(locations, up_left);
        self.can_move_line(locations, down_right);
        self.can_move_line(locations, down_left);
    }

    fn can_move_line(&self, locations: &mut Vec<Action>, line: impl Iterator<Item = Rel>) {
        for rel in line {
            match self.can_move_to(rel) {
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

    fn can_move_to(&self, rel: Rel) -> Option<MoveType> {
        match self.get(rel) {
            Some(Square::Empty) => Some(MoveType::Move(rel)),
            Some(Square::Piece(to_color, _)) => {
                if self.game.turn() != to_color {
                    Some(MoveType::Take(rel))
                } else {
                    None
                }
            }
            Some(Square::Duck) => None,
            None => None,
        }
    }

    pub fn add_if_movable(&self, rel: Rel, locations: &mut Vec<Action>) {
        if let Some(move_type) = self.can_move_to(rel) {
            locations.push(Action::Move(move_type.get()));
        }
    }

    pub fn castle_move(&self, king_moved: bool, locations: &mut Vec<Action>) {
        if !king_moved {
            for side in Side::all() {
                if let Some(Square::Piece(_, Piece::Rook { moved: false })) = self.get(side.rook())
                {
                    if Rel::path_to(side.dir() * 2).all(|rel| self.get(rel) == Some(Square::Empty))
                    {
                        locations.push(Action::Castle(side));
                    }
                }
            }
        }
    }

    pub fn forward_pawn(&self, loc: Loc, locations: &mut Vec<Action>) {
        let home = match self.game.turn() {
            Color::Black => 1,
            Color::White => 6,
        };

        let color = self.game.turn();
        let one_step = Rel::new(0, color.dir());
        self.add_if_no_take(one_step, locations);
        if loc.down == home {
            if let Some(Square::Empty) = self.get(one_step) {
                self.add_if_no_take(Rel::new(0, color.dir() * 2), locations);
            }
        }
    }

    pub fn capture_pawn(&self, locations: &mut Vec<Action>) {
        let color = self.game.turn();
        self.add_if_takable(Rel::new(1, color.dir()), locations);
        self.add_if_takable(Rel::new(-1, color.dir()), locations);
    }

    fn add_if_takable(&self, rel: Rel, locations: &mut Vec<Action>) {
        if let Some(Square::Piece(to_color, _)) = self.get(rel) {
            if self.game.turn() != to_color {
                locations.push(Action::Move(rel))
            }
        }
    }

    fn add_if_no_take(&self, rel: Rel, locations: &mut Vec<Action>) {
        if let Some(Square::Empty) = self.get(rel) {
            locations.push(Action::Move(rel))
        }
    }

    pub fn en_passant(&self, locations: &mut Vec<Action>) {
        for side in Side::all() {
            if let Some(Square::Piece(other_color, Piece::Pawn { passantable })) =
                self.get(side.dir())
            {
                if self.game.turn() != other_color && passantable {
                    locations.push(Action::EnPassant(side));
                }
            }
        }
    }

    pub fn mk_promotions(&self, loc: Loc, actions: &mut Vec<Action>) {
        let mut new_actions = Vec::new();
        for action in actions.iter() {
            if let Action::Move(rel) = action {
                let new_down = rel.from(loc).down;
                if new_down == 0 || new_down == 7 {
                    for square in self.game.mk_promotion_board().squares() {
                        let action = Action::Promote(*rel, *square);
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
