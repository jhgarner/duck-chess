use crate::{board::BoardFocus, *};
use anyhow::{bail, Result};
use once_cell::sync::Lazy;

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    pub maker: Player,
    pub joiner: Player,
    pub board: Board,
    pub turns: Vec<Turn>,
    pub maker_color: Color,
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

        if self.maker.id == player.id {
            result.push(self.maker_color);
        }
        if self.joiner.id == player.id {
            result.push(self.maker_color.other());
        }

        result
    }

    pub fn get(&self, loc: Loc) -> Option<Square> {
        self.board
            .grid
            .get(loc.down)
            .and_then(|row| row.get(loc.right))
            .copied()
    }

    pub fn get_mut(&mut self, loc: Loc) -> &mut Square {
        &mut self.board.grid[loc.down][loc.right]
    }

    pub fn mk_promotion_board(&self) -> &'static Board {
        static WHITE_BOARD: Lazy<Board> = Lazy::new(|| Board {
            grid: vec![vec![
                Square::Piece(Color::White, Piece::Queen),
                Square::Piece(Color::White, Piece::Knight),
                Square::Piece(Color::White, Piece::Rook { moved: true }),
                Square::Piece(Color::White, Piece::Bishop),
            ]],
        });
        static BLACK_BOARD: Lazy<Board> = Lazy::new(|| Board {
            grid: vec![vec![
                Square::Piece(Color::Black, Piece::Queen),
                Square::Piece(Color::Black, Piece::Knight),
                Square::Piece(Color::Black, Piece::Rook { moved: true }),
                Square::Piece(Color::Black, Piece::Bishop),
            ]],
        });

        if let Color::White = self.turn() {
            &*WHITE_BOARD
        } else {
            &*BLACK_BOARD
        }
    }

    pub fn valid_duck(&self, loc: Loc) -> bool {
        self.get(loc) == Some(Square::Empty)
    }

    pub fn apply_duck(&mut self, loc: Loc) {
        for square in self.board.squares_mut() {
            if let Square::Duck = square {
                *square = Square::Empty;
            }
        }
        *self.get_mut(loc) = Square::Duck;
    }

    pub fn apply(&mut self, loc: Loc, action: Action) {
        for square in self.board.squares_mut() {
            square.unpassant_pawns();
        }

        let mut board = BoardFocus::new(self);
        board.focus(loc);
        match action {
            Action::Move(rel) => {
                board.move_to(rel);
            }
            Action::Castle(side) => {
                board.move_to(side.dir() * 2);
                board.shift(side.rook()).move_to(side.rook_to());
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

    pub fn valid_locations(&self, loc: Loc) -> Vec<Action> {
        let mut board = BoardFocus::new(self);
        let mut locations: Vec<Action> = Vec::new();
        if let Square::Piece(color, piece) = board.focus(loc).focused() {
            if color != self.turn() {
                return Vec::new();
            }
            match piece {
                Piece::King { moved } => {
                    board.move_with_adj(&mut locations, 1);
                    board.move_with_diag(&mut locations, 1);
                    board.castle_move(moved, &mut locations);
                }
                Piece::Queen => {
                    board.move_with_adj(&mut locations, 8);
                    board.move_with_diag(&mut locations, 8);
                }
                Piece::Rook { .. } => {
                    board.move_with_adj(&mut locations, 8);
                }
                Piece::Bishop => {
                    board.move_with_diag(&mut locations, 8);
                }
                Piece::Knight => {
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
                        board.add_if_movable(rel, &mut locations)
                    }
                }
                Piece::Pawn { .. } => {
                    board.forward_pawn(loc, &mut locations);
                    board.capture_pawn(&mut locations);
                    board.en_passant(&mut locations);
                    board.mk_promotions(loc, &mut locations);
                }
            }
        }

        locations
    }

    pub fn is_player_turn(&self, player: &Player) -> bool {
        self.player(player).contains(&self.turn())
    }

    pub fn apply_turn(&mut self, player: &Player, turn: Turn) -> Result<()> {
        if self.is_player_turn(player) {
            let actions = self.valid_locations(turn.from);
            if let Some(action) = actions.iter().find(|action| *action == &turn.action) {
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
