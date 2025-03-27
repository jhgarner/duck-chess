use std::collections::{HashMap, HashSet};

use crate::*;
use anyhow::{Context, Result, bail};
use boardfocus::BoardFocus;
use hexboard::Hexboard;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub maker: Player,
    pub joiner: Player,
    pub some_game: SomeGame,
    pub maker_color: Color,
}

impl Game {
    pub fn mk_promotion_pieces() -> [Piece; 4] {
        [
            Piece::Queen,
            Piece::Knight,
            Piece::Rook { moved: true },
            Piece::Bishop,
        ]
    }

    pub fn player(&self, player: &Player) -> PlayerColor {
        let mut result = PlayerColor::default();

        if self.maker.id == player.id {
            result = result.push(self.maker_color);
        }
        if self.joiner.id == player.id {
            result = result.push(self.maker_color.other());
        }

        result
    }

    pub fn game_over(&self) -> Option<Color> {
        match &self.some_game {
            SomeGame::Square(game) => game.game_over(),
            SomeGame::Hex(game) => game.game_over(),
        }
    }

    pub fn turn(&self) -> Color {
        match &self.some_game {
            SomeGame::Square(game) => game.turn(),
            SomeGame::Hex(game) => game.turn(),
        }
    }

    pub fn is_player_turn(&self, player: &Player) -> bool {
        self.player(player).contains(&self.turn())
    }

    pub fn apply_turn(&mut self, player: &Player, turn: SomeTurn) -> Result<()> {
        if !self.is_player_turn(player) {
            bail!("Not your turn")
        }
        match (&mut self.some_game, turn) {
            (SomeGame::Square(game), SomeTurn::Square(turn)) => game.apply_turn(turn),
            (SomeGame::Hex(game), SomeTurn::Hex(turn)) => game.apply_turn(turn),
            _ => bail!("Turn and game did not match!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SomeGame {
    Square(GameRaw<Board>),
    Hex(GameRaw<Hexboard>),
}

impl From<GameRaw<Board>> for SomeGame {
    fn from(value: GameRaw<Board>) -> Self {
        Self::Square(value)
    }
}

impl From<GameRaw<Hexboard>> for SomeGame {
    fn from(value: GameRaw<Hexboard>) -> Self {
        Self::Hex(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SomeBoard<'a> {
    Square(&'a Board),
    Hex(&'a Hexboard),
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub enum GameTypes {
    Square,
    Hex,
}

impl GameTypes {
    pub fn mk_game(&self, maker: Player, joiner: Player, maker_color: Color) -> Game {
        match self {
            GameTypes::Square => Game {
                maker,
                joiner,
                maker_color,
                some_game: SomeGame::Square(GameRaw {
                    board: Board::default(),
                    turns: Vec::new(),
                    duck_loc: None,
                }),
            },
            GameTypes::Hex => Game {
                maker,
                joiner,
                maker_color,
                some_game: SomeGame::Hex(GameRaw {
                    board: Hexboard::default(),
                    turns: Vec::new(),
                    duck_loc: None,
                }),
            },
        }
    }
}

impl SomeGame {}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRaw<Board: ChessBoard> {
    pub board: Board,
    pub turns: Vec<TurnRaw<Board>>,
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
            .map(|piece| Square::Piece(self.turn(), *piece, SquareId::default()))
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

    pub fn game_over(&self) -> Option<Color> {
        let (mut found_white, mut found_black) = (false, false);
        for (_, square) in self.board.iter() {
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

    pub fn apply_turn(&mut self, turn: TurnRaw<Board>) -> Result<()> {
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
}
