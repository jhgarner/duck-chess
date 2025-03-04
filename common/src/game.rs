use std::collections::{HashMap, HashSet};

use crate::{board::ChessBoard, *};
use anyhow::{Result, bail};

pub type Game = GameRaw<Board>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SomeGame {
    Square(GameRaw<Board>),
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub enum GameTypes {
    Square,
}

impl GameTypes {
    pub fn mk_game(&self, maker: Player, joiner: Player, maker_color: Color) -> SomeGame {
        match self {
            GameTypes::Square => SomeGame::Square(GameRaw {
                maker,
                joiner,
                maker_color,
                board: Board::default(),
                turns: Vec::new(),
                duck_loc: None,
            }),
        }
    }
}

impl SomeGame {
    pub fn maker(&self) -> &Player {
        match self {
            SomeGame::Square(game) => &game.maker,
        }
    }

    pub fn joiner(&self) -> &Player {
        match self {
            SomeGame::Square(game) => &game.joiner,
        }
    }

    pub fn board(&self) -> &impl ChessBoard {
        match self {
            SomeGame::Square(game) => &game.board,
        }
    }
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRaw<Board: ChessBoard> {
    pub maker: Player,
    pub joiner: Player,
    pub board: Board,
    pub turns: Vec<TurnRaw<Board>>,
    pub maker_color: Color,
    pub duck_loc: Option<Board::Loc>,
}

impl<Board: ChessBoard> GameRaw<Board> {
    pub fn turn(&self) -> Color {
        if self.turns.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn is_player_turn(&self, player: &Player) -> bool {
        self.player(player).contains(&self.turn())
    }

    pub fn player(&self, player: &Player) -> Vec<Color> {
        let mut result = Vec::new();

        if self.maker.id == player.id {
            result.push(self.maker_color);
        }
        if self.joiner.id == player.id {
            result.push(self.maker_color.other());
        }

        result
    }

    pub fn mk_promotion_pieces() -> [Piece; 4] {
        [
            Piece::Queen,
            Piece::Knight,
            Piece::Rook { moved: true },
            Piece::Bishop,
        ]
    }

    pub fn get(&self, loc: Board::Loc) -> Option<Square> {
        self.board.get(loc)
    }

    pub fn valid_duck(&self, loc: Board::Loc) -> bool {
        self.get(loc) == Some(Square::Empty)
    }

    pub fn valid_locations(&self, loc: Board::Loc) -> HashMap<Board::Loc, ActionRaw<Board::Rel>> {
        if let Some(Square::Piece(color, piece)) = self.board.get(loc) {
            if color != self.turn() {
                HashMap::new()
            } else {
                self.board.valid_locations(color, piece, loc)
            }
        } else {
            HashMap::new()
        }
    }

    pub fn mk_small_board(&self, pieces: &[Piece]) -> Board {
        let squares = pieces
            .iter()
            .map(|piece| Square::Piece(self.turn(), *piece))
            .collect();
        Board::mk_promotion_board(squares)
    }

    pub fn empties(&self) -> HashSet<Board::Loc> {
        self.board
            .iter()
            .filter_map(|(loc, square)| {
                if let Square::Empty = square {
                    Some(loc)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn empty_board() -> GameRaw<Board> {
        Self {
            maker: Player {
                ..Default::default()
            },
            joiner: Player {
                ..Default::default()
            },
            maker_color: Color::White,
            turns: Vec::new(),
            board: Board::default(),
            duck_loc: None,
        }
    }
}

impl<Board: ChessBoard> GameRaw<Board> {
    pub fn get_mut(&mut self, loc: Board::Loc) -> &mut Square {
        self.board.get_mut(loc).unwrap()
    }

    pub fn apply_duck(&mut self, loc: Board::Loc) {
        if let Some(duck_loc) = self.duck_loc {
            *self.get_mut(duck_loc) = Square::Empty;
        }
        self.duck_loc = Some(loc);
        *self.get_mut(loc) = Square::Duck;
    }

    pub fn apply(&mut self, loc: Board::Loc, action: SingleAction<Board::Rel>) {
        self.board.apply(loc, self.turn(), action)
    }
}

impl Game {
    pub fn apply_turn(&mut self, player: &Player, turn: Turn) -> Result<()> {
        if self.is_player_turn(player) {
            let actions = self.valid_locations(turn.from);
            if actions.values().any(|action| action.contains(&turn.action)) {
                self.apply(turn.from, turn.action);
                if self.valid_duck(turn.duck_to) {
                    self.apply_duck(turn.duck_to);
                    self.turns.push(turn);
                    return Ok(());
                }
                bail!("Invalid Duck")
            }
            bail!("Invalid Action {:?} {:?}", turn.action, actions)
        }
        bail!("Invalid Game")
    }

    pub fn game_over(&self) -> Option<Color> {
        let (mut found_white, mut found_black) = (false, false);
        for square in self.board.squares() {
            found_white |= square.is_king(Color::White);
            found_black |= square.is_king(Color::Black);
        }

        if !found_white {
            Some(Color::Black)
        } else if !found_black {
            Some(Color::White)
        } else {
            None
        }
    }
}
