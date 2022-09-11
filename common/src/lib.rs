use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    ops::{Add, Deref, DerefMut, Mul},
};

pub mod board;
pub mod game;

pub use board::Board;
pub use game::Game;

// TODO How do I feel about having so much random junk in this file? It started as a bunch of
// structs, then those structs got implementations, then I pulled out a couple really big ones, but
// it still feels messy. How best to organize this?

// TODO It would be cool to make WithId<T> replace having the Id field on everything! Then I don't
// need to use Option<Id> anywhere because the type encodes it. I tried changing everything, but it
// got a little gross...
#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct GameRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub maker: Player,
}

// TBH I don't really want to expose this wrapper type, but I have use it on the backend to
// disambiguate between the completed and noncompleted databases. If I #flatten it then I can drop
// the id field and it'll look the same as Game in memory. I should probably move this to the
// backend crate...
#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct CompletedGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub game: Game,
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct MyGames {
    pub my_turn: Vec<Game>,
    pub other_turn: Vec<Game>,
    pub unstarted: Vec<GameRequest>,
    pub completed: Vec<Game>,
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Turn {
    pub from: Loc,
    pub action: Action,
    pub duck_to: Loc,
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    King,
    Queen,
}

impl Side {
    pub fn all() -> [Side; 2] {
        use Side::*;
        [King, Queen]
    }

    pub fn rook(&self) -> Rel {
        match self {
            Side::King => Rel::new(3, 0),
            Side::Queen => Rel::new(-4, 0),
        }
    }

    pub fn rook_to(&self) -> Rel {
        match self {
            Side::King => Rel::new(-2, 0),
            Side::Queen => Rel::new(3, 0),
        }
    }

    pub fn dir(&self) -> Rel {
        match self {
            Side::King => Rel::new(1, 0),
            Side::Queen => Rel::new(-1, 0),
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Move(Rel),
    EnPassant(Side),
    Promote(Rel, Square),
    Castle(Side),
}

impl Action {
    pub fn get_target(&self, game: &Game) -> Rel {
        match *self {
            Action::Move(rel) => rel,
            Action::EnPassant(side) => Rel::new(side.dir().right, game.turn().dir()),
            Action::Castle(side) => side.dir() * 2,
            Action::Promote(rel, _) => rel,
        }
    }
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

    pub fn all() -> [Piece; 6] {
        [
            Piece::King { moved: false },
            Piece::Queen,
            Piece::Knight,
            Piece::Rook { moved: false },
            Piece::Bishop,
            Piece::Pawn { passantable: false },
        ]
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
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Square {
    Empty,
    Duck,
    Piece(Color, Piece),
}

impl Square {
    pub fn unpassant_pawns(&mut self) {
        if let Square::Piece(color, Piece::Pawn { .. }) = self {
            *self = Square::Piece(*color, Piece::Pawn { passantable: false });
        }
    }

    pub fn moves(self, rel: Rel) -> Self {
        match self {
            Square::Piece(color, Piece::King { .. }) => {
                Square::Piece(color, Piece::King { moved: true })
            }
            Square::Piece(color, Piece::Rook { .. }) => {
                Square::Piece(color, Piece::Rook { moved: true })
            }
            Square::Piece(color, Piece::Pawn { .. }) => {
                if rel.down.abs() == 2 {
                    Square::Piece(color, Piece::Pawn { passantable: true })
                } else {
                    Square::Piece(color, Piece::Pawn { passantable: false })
                }
            }
            piece => piece,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Square::Piece(color, piece) => format!("{}{}", color.short_name(), piece.short_name()),
            Square::Duck => "duck".into(),
            Square::Empty => "empty".into(),
        }
    }

    pub fn is_king(&self, color: Color) -> bool {
        if let Square::Piece(piece_color, Piece::King { .. }) = self {
            *piece_color == color
        } else {
            false
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    pub fn from(&self, loc: Loc) -> Loc {
        let down = (loc.down as i32 + self.down) as usize;
        let right = (loc.right as i32 + self.right) as usize;
        Loc::new(right, down)
    }

    pub fn shift(&self, right: i32, down: i32) -> Rel {
        Rel::new(self.right + right, self.down + down)
    }

    pub fn path_to(to: Rel) -> impl Iterator<Item = Rel> {
        let right_step = match to.right.cmp(&0) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
        };

        let down_step = match to.down.cmp(&0) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
        };

        RelPath {
            from: Rel::origin(),
            to,
            step: Rel::new(right_step, down_step),
        }
    }
}

impl Add<Rel> for Rel {
    type Output = Rel;
    fn add(self, rhs: Rel) -> Self::Output {
        Rel::new(self.right + rhs.right, self.down + rhs.down)
    }
}

impl Mul<i32> for Rel {
    type Output = Rel;
    fn mul(self, rhs: i32) -> Self::Output {
        Rel::new(self.right * rhs, self.down * rhs)
    }
}

struct RelPath {
    from: Rel,
    to: Rel,
    step: Rel,
}

impl Iterator for RelPath {
    type Item = Rel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.from == self.to {
            None
        } else {
            self.from = self.from + self.step;
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
