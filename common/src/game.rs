use std::collections::HashMap;

use crate::{
    board::{ChessBoard, ChessBoardMut},
    *,
};
use anyhow::{bail, Result};

pub type Game = GameRaw<Board>;

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
        self.board.get(loc).copied()
    }

    pub fn valid_duck(&self, loc: Board::Loc) -> bool {
        self.get(loc) == Some(Square::Empty)
    }

    pub fn valid_locations(&self, loc: Board::Loc) -> HashMap<Board::Loc, ActionRaw<Board::Rel>> {
        if let Some(&Square::Piece(color, piece)) = self.board.get(loc) {
            if color != self.turn() {
                HashMap::new()
            } else {
                self.board.valid_locations(color, piece, loc)
            }
        } else {
            HashMap::new()
        }
    }

    pub fn mk_promotion_board(&self) -> Board::Board {
        let pieces = Game::mk_promotion_pieces()
            .into_iter()
            .map(|piece| Square::Piece(self.turn(), piece))
            .collect();
        Board::mk_promotion_board(pieces)
    }
}

impl<Board: ChessBoardMut> GameRaw<Board> {
    pub fn get_mut(&mut self, loc: Board::Loc) -> &mut Square {
        self.board.get_mut(loc).unwrap()
    }

    pub fn apply_duck(&mut self, loc: Board::Loc) {
        if let Some(duck_loc) = self.duck_loc {
            *self.get_mut(duck_loc) = Square::Empty;
        }
        *self.get_mut(loc) = Square::Duck;
    }

    pub fn apply(&mut self, loc: Board::Loc, action: ActionRaw<Board::Rel>) {
        self.board.apply(loc, self.turn(), action)
    }
}

impl Game {
    pub fn apply_turn(&mut self, player: &Player, turn: Turn) -> Result<()> {
        if self.is_player_turn(player) {
            let actions = self.valid_locations(turn.from);
            if let Some(action) = actions.values().find(|action| *action == &turn.action) {
                self.apply(turn.from, *action);
                if self.valid_duck(turn.duck_to) {
                    self.apply_duck(turn.duck_to);
                    self.turns.push(turn);
                    return Ok(());
                }
            }
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
