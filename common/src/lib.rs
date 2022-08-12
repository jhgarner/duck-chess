use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    ops::{Add, Deref, DerefMut, Mul},
};

pub mod rules;

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default)]
pub struct WithId<T> {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    t: T,
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

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub white: Player,
    pub black: Player,
    pub board: Board,
    pub turns: Vec<Turn>,
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct CompletedGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub game: Game
}

impl Game {
    pub fn turn(&self) -> Color {
        if self.turns.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn player(&self, player: &Player) -> Vec<Color> {
        let mut result = Vec::new();

        if self.white.id == player.id {
            result.push(Color::White);
        }
        if self.black.id == player.id {
            result.push(Color::Black);
        }

        result
    }

    pub fn get(&self, loc: Loc) -> Option<Square> {
        self.board
            .0
            .get(loc.down)
            .and_then(|row| row.get(loc.right))
            .copied()
    }

    pub fn get_mut(&mut self, loc: Loc) -> &mut Square {
        &mut self.board.0[loc.down][loc.right]
    }

    pub fn squares(&self) -> impl Iterator<Item = &Square> {
        self.board.0.iter().flat_map(|row| row.iter())
    }

    pub fn squares_mut(&mut self) -> impl Iterator<Item = &mut Square> {
        self.board.0.iter_mut().flat_map(|row| row.iter_mut())
    }

    pub fn rows(&self) -> impl Iterator<Item = &[Square; 8]> {
        self.board.0.iter()
    }
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct MyGames {
    pub started: Vec<Game>,
    pub unstarted: Vec<GameRequest>,
    pub completed: Vec<CompletedGame>,
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Board(pub [[Square; 8]; 8]);

impl Board {}

impl Default for Board {
    fn default() -> Self {
        use Piece::*;

        let rook = Rook { moved: false };
        let king = King { moved: false };
        let back = [rook, Knight, Bishop, Queen, king, Bishop, Knight, rook];
        let front = [Pawn { passantable: false }; 8];
        let empty = [Square::Empty; 8];
        let board = [
            back.map(|piece| Square::Piece(Color::Black, piece)),
            front.map(|piece| Square::Piece(Color::Black, piece)),
            empty,
            empty,
            empty,
            empty,
            front.map(|piece| Square::Piece(Color::White, piece)),
            back.map(|piece| Square::Piece(Color::White, piece)),
        ];
        Board(board)
    }
}

#[derive(Debug, Hash, Copy, Clone, Serialize, Deserialize)]
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

    pub fn king_to(&self) -> Rel {
        match self {
            Side::King => Rel::new(2, 0),
            Side::Queen => Rel::new(-3, 0),
        }
    }

    pub fn rook(&self) -> Rel {
        match self {
            Side::King => Rel::new(3, 0),
            Side::Queen => Rel::new(-4, 0),
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
            Action::Castle(side) => side.king_to(),
            Action::Promote(rel, _) => rel,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq)]
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

impl Hash for Piece {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.short_name().hash(state);
    }
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        self.short_name() == other.short_name()
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
        let right_step = if to.right == 0 {
            0
        } else if to.right > 0 {
            1
        } else {
            -1
        };

        let down_step = if to.down == 0 {
            0
        } else if to.down > 0 {
            1
        } else {
            -1
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
        Rel::new(self.right * rhs, self.down + rhs)
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
