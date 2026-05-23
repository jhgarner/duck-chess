use bson::oid::ObjectId;
pub use chessboard::ChessBoard;
use game::GameTypes;
use hexboard::Hexboard;
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    hash::Hash,
    iter,
    ops::{Add, Deref, DerefMut, Mul},
};

pub mod board;
mod boardfocus;
pub mod chessboard;
pub mod events;
pub mod game;
pub mod hexboard;
pub mod hexgame;
pub mod menuboard;

pub use board::Board;
pub use game::Game;

use crate::board::game::SomeLoc;

// TODO How do I feel about having so much random junk in this file? It started as a bunch of
// structs, then those structs got implementations, then I pulled out a couple really big ones, but
// it still feels messy. How best to organize this?

// TODO It would be cool to make WithId<T> replace having the Id field on everything! Then I don't
// need to use Option<Id> anywhere because the type encodes it. I tried changing everything, but it
// got a little gross...
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WithId<T> {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub t: T,
}

impl<T> WithId<T> {
    pub fn new(id: ObjectId, t: T) -> WithId<T> {
        WithId { id, t }
    }
}

impl<T> Deref for WithId<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T> DerefMut for WithId<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.t
    }
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default)]
pub struct PasswordPlayer {
    pub password: String,
    #[serde(flatten)]
    pub player: Player,
}

impl Deref for PasswordPlayer {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.player
    }
}

impl DerefMut for PasswordPlayer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.player
    }
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Player {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRequest {
    pub game_type: GameTypes,
    pub maker: Player,
}

impl PartialEq for GameRequest {
    fn eq(&self, _: &Self) -> bool {
        // Used in the frontend to force rerendering
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnyGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub game: GameOrRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameOrRequest {
    Game(Game),
    Completed(Game),
    Request(GameRequest),
}

impl GameOrRequest {
    pub fn in_game(&self, player: &Player) -> bool {
        match self {
            GameOrRequest::Game(game) => player == &game.maker || player == &game.joiner,
            GameOrRequest::Completed(game) => player == &game.maker || player == &game.joiner,
            GameOrRequest::Request(_) => true,
        }
    }

    pub fn pieces(&self) -> Box<dyn Iterator<Item = (SomeLoc, Square)> + '_> {
        enum Iter {
            Square,
        }
        match self {
            Self::Game(Game { some_game, .. }) => some_game.iter(),
            Self::Completed(Game { some_game, .. }) => some_game.iter(),
            Self::Request(_) => Box::new(iter::empty()),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum SomeTurn {
    Square(TurnRaw<Board>),
    Hex(TurnRaw<Hexboard>),
}

#[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnRaw<Board: ChessBoard> {
    pub from: Board::Loc,
    pub action: SingleAction<Board::Rel>,
    pub duck_to: Board::Loc,
}

impl<Board: ChessBoard> Clone for TurnRaw<Board> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Board: ChessBoard> Copy for TurnRaw<Board> {}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionRaw<Rel> {
    Just(SingleAction<Rel>),
    Promotion(Rel, Vec<Piece>),
}

impl<Rel: PartialEq> ActionRaw<Rel> {
    pub fn contains(&self, item: &SingleAction<Rel>) -> bool {
        match self {
            Self::Just(action) => action == item,
            Self::Promotion(prom_rel, options) => {
                if let SingleAction::Move(mov_rel, piece) = item {
                    prom_rel == mov_rel && options.contains(piece)
                } else {
                    false
                }
            }
        }
    }

    pub fn move_it(rel: Rel, to: Piece) -> ActionRaw<Rel> {
        Self::Just(SingleAction::Move(rel, to))
    }

    pub fn en_passant(rel: Rel) -> ActionRaw<Rel> {
        Self::Just(SingleAction::EnPassant(rel))
    }

    pub fn castle(castle: Castle<Rel>) -> ActionRaw<Rel> {
        Self::Just(SingleAction::Castle(castle))
    }
}

#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Castle<Rel> {
    rook: Rel,
    rook_to: Rel,
    steps: RelIter<Rel>,
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SingleAction<Rel> {
    Move(Rel, Piece),
    EnPassant(Rel),
    Castle(Castle<Rel>),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct UniquePiece {
    piece: Piece,
    id: u16,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Piece {
    King { moved: bool },
    Queen,
    Bishop,
    Knight,
    Rook { moved: bool },
    Pawn { passantable: bool },
}

impl Piece {
    pub fn short_name(&self) -> char {
        use Piece::*;

        match self {
            King { .. } => 'K',
            Queen => 'Q',
            Bishop => 'B',
            Knight => 'N',
            Rook { .. } => 'R',
            Pawn { .. } => 'P',
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn short_name(&self) -> char {
        use Color::*;

        match self {
            Black => 'b',
            White => 'w',
        }
    }

    pub fn dir(&self) -> i32 {
        match self {
            Color::Black => 1,
            Color::White => -1,
        }
    }

    pub fn other(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub fn all() -> impl IntoIterator<Item = Color> {
        [Color::White, Color::Black]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PlayerColor {
    #[default]
    None,
    White,
    Black,
    Both,
}

impl From<Color> for PlayerColor {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => Self::Black,
            Color::White => Self::White,
        }
    }
}

impl PlayerColor {
    pub fn push(self, rhs: Color) -> Self {
        match self {
            Self::None => rhs.into(),
            Self::Both => Self::Both,
            _ => {
                if self != rhs.into() {
                    Self::Both
                } else {
                    self
                }
            }
        }
    }

    pub fn contains(self, target: &Color) -> bool {
        match self {
            Self::None => false,
            Self::Both => true,
            _ => self == (*target).into(),
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquareId(pub u16);

impl Default for SquareId {
    fn default() -> Self {
        thread_local! {
            static NEXT: Cell<u16> = Cell::new(0);
        }

        Self(NEXT.replace(NEXT.get().wrapping_add(1)))
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Square {
    Empty,
    Duck,
    Piece(Color, Piece, #[serde(default)] SquareId),
}

impl Square {
    pub fn piece(color: Color, piece: Piece, id_counter: &mut u16) -> Self {
        *id_counter += 1;
        Self::Piece(color, piece, SquareId(*id_counter))
    }

    pub fn unpassant_pawns(&mut self) {
        if let Square::Piece(color, Piece::Pawn { .. }, id) = self {
            *self = Square::Piece(*color, Piece::Pawn { passantable: false }, *id);
        }
    }

    pub fn name(&self) -> String {
        match self {
            Square::Piece(color, piece, _) => {
                format!("{}{}", color.short_name(), piece.short_name())
            }
            Square::Duck => "duck".into(),
            Square::Empty => "empty".into(),
        }
    }

    pub fn is_king(&self, color: Color) -> bool {
        if let Square::Piece(piece_color, Piece::King { .. }, _) = self {
            *piece_color == color
        } else {
            false
        }
    }

    pub fn get_piece(self) -> Option<(Color, Piece, SquareId)> {
        if let Self::Piece(color, piece, id) = self {
            Some((color, piece, id))
        } else {
            None
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Loc {
    pub right: usize,
    pub down: usize,
}

impl Loc {
    pub fn new(right: impl TryInto<usize>, down: impl TryInto<usize>) -> Loc {
        Loc {
            right: right.try_into().ok().unwrap(),
            down: down.try_into().ok().unwrap(),
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rel {
    pub right: i32,
    pub down: i32,
}

impl Rel {
    pub fn new(right: impl TryInto<i32>, down: impl TryInto<i32>) -> Rel {
        Rel {
            right: right.try_into().ok().unwrap(),
            down: down.try_into().ok().unwrap(),
        }
    }

    pub fn origin() -> Rel {
        Rel { right: 0, down: 0 }
    }

    pub fn shift(&self, right: i32, down: i32) -> Rel {
        Rel::new(self.right + right, self.down + down)
    }
}

impl Add<Rel> for Rel {
    type Output = Rel;
    fn add(self, rhs: Rel) -> Self::Output {
        Rel::new(self.right + rhs.right, self.down + rhs.down)
    }
}

impl Add<Rel> for Loc {
    type Output = Loc;
    fn add(self, rhs: Rel) -> Self::Output {
        let down = (self.down as i32 + rhs.down) as usize;
        let right = (self.right as i32 + rhs.right) as usize;
        Loc::new(right, down)
    }
}

impl Mul<i32> for Rel {
    type Output = Rel;
    fn mul(self, rhs: i32) -> Self::Output {
        Rel::new(self.right * rhs, self.down * rhs)
    }
}

#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
struct RelIter<Rel> {
    from: Rel,
    fuel: u32,
    step: Rel,
}

impl<Rel: Copy + Default> RelIter<Rel> {
    pub fn new(start: Rel, fuel: u32) -> Self {
        RelIter {
            from: Rel::default(),
            fuel,
            step: start,
        }
    }
}

impl<Rel: Mul<i32, Output = Rel> + Add<Rel, Output = Rel>> RelIter<Rel> {
    pub fn total(self) -> Rel {
        self.from + self.step * self.fuel as i32
    }
}

impl<Rel: Copy + PartialEq + Add<Rel, Output = Rel>> Iterator for RelIter<Rel> {
    type Item = Rel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fuel == 0 {
            None
        } else {
            self.from = self.from + self.step;
            self.fuel -= 1;
            Some(self.from)
        }
    }
}

// trait Filterable {
//     type Target;

//     fn filter_doc(self) -> Document;
//     fn from_target(from: &Self::Target) -> Self;
// }

// // struct PlayerBuilder {
// //     pub id: Option<Option<ObjectId>>,
// //     pub name: Option<String>,
// // }

// // impl Filterable for PlayerBuilder {
// //     type Target = Player;

// //     fn filter_doc(self) -> Document {
// //         let mut doc = Document::new();
// //         if let Some(id) = self.id {
// //             doc.insert("id", )
// //         }
// //         todo!()
// //     }

// //     fn from_target(from: &Self::Target) -> Self {
// //         todo!()
// //     }

// // }
