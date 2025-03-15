use std::collections::{HashMap, HashSet};

use crate::*;
use anyhow::{Context, Result, bail};
use boardfocus::BoardFocus;

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

    pub fn valid_locations_from(
        &self,
        loc: Board::Loc,
    ) -> HashMap<Board::Loc, ActionRaw<Board::Rel>> {
        if let Some(board) = BoardFocus::new(&self.board, loc) {
            board.valid_locations_for(self.turn())
        } else {
            HashMap::new()
        }
    }

    pub fn mk_squares_for(&self, pieces: &[Piece]) -> Vec<Square> {
        pieces
            .iter()
            .map(|piece| Square::Piece(self.turn(), *piece))
            .collect()
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

    pub fn apply_from(&mut self, loc: Board::Loc, action: SingleAction<Board::Rel>) {
        if let Some(mut board) = BoardFocus::new(&mut self.board, loc) {
            board.apply(action)
        }
    }
}

impl Game {
    pub fn apply_turn(&mut self, player: &Player, turn: Turn) -> Result<()> {
        if !self.is_player_turn(player) {
            bail!("Not your turn")
        }
        let turn_color = self.turn();
        let mut board = BoardFocus::new(&mut self.board, turn.from).context("Invalid board")?;
        let actions = board.valid_locations_for(turn_color);
        if !actions.values().any(|action| action.contains(&turn.action)) {
            bail!("Invalid Action {:?} {:?}", turn.action, actions)
        }
        board.apply(turn.action);
        if !self.valid_duck(turn.duck_to) {
            bail!("Invalid Duck")
        }
        self.apply_duck(turn.duck_to);
        self.turns.push(turn);
        Ok(())
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
