use std::collections::HashMap;

use common::board::ChessBoard;
use common::game::GameRaw;

use crate::board::{self};
use crate::{board::Active, prelude::*};

#[derive(PartialEq, Eq, Clone, Default)]
pub enum GameState<Board: ChessBoard> {
    #[default]
    Waiting,
    MyMove,
    Selected(Board::Loc, HashMap<Board::Loc, ActionRaw<Board::Rel>>),
    Promotion(Board::Loc, Board::Rel, Vec<Piece>),
    PlacingDuck(Board::Loc, SingleAction<Board::Rel>),
}

#[component]
pub fn active_game(id: ObjectId, og_game: GameRaw<Board>) -> Element {
    let game = use_signal(|| og_game.clone());
    let game_state = use_signal(|| GameState::MyMove);

    let (board, active, targets): (_, _, HashSet<_>) = match game_state() {
        GameState::Selected(start, actions) => {
            let targets = actions.keys().copied().collect();
            (game.read().board.clone(), Active::Active(start), targets)
        }
        GameState::Promotion(_, _, pieces) => (
            game.read().mk_small_board(&pieces),
            Active::NoActive,
            HashSet::new(),
        ),
        GameState::PlacingDuck(_, _) => {
            let targets = game.read().empties();
            let duck = game.read().duck_loc.into();
            (game.read().board.clone(), duck, targets)
        }
        GameState::MyMove | GameState::Waiting => {
            (game.read().board.clone(), Active::NoActive, HashSet::new())
        }
    };

    rsx! {
        board::BoardC {
            action: move |loc| {
                update(id, game, game_state, loc)
            },
            board: board,
            active: active,
            targets: targets,
        }
    }
}

fn update<Board: ChessBoard>(
    id: ObjectId,
    mut game: Signal<GameRaw<Board>>,
    mut game_state: Signal<GameState<Board>>,
    loc: Board::Loc,
) {
    let new_game_state = match game_state.take() {
        GameState::Waiting => GameState::Waiting,
        GameState::MyMove => {
            let valid_moves = game.read().valid_locations(loc);
            if valid_moves.is_empty() {
                GameState::MyMove
            } else {
                GameState::Selected(loc, valid_moves)
            }
        }
        GameState::Selected(start, mut valid_moves) => {
            if let Some(action) = valid_moves.remove(&loc) {
                match action {
                    ActionRaw::Promotion(rel, options) => GameState::Promotion(start, rel, options),
                    ActionRaw::Just(action) => {
                        game.write().apply(start, action);
                        GameState::PlacingDuck(start, action)
                    }
                }
            } else {
                GameState::MyMove
            }
        }
        GameState::Promotion(start, rel, options) => {
            let square = game.read().mk_small_board(&options).get(loc);
            if let Some(Square::Piece(_, piece)) = square {
                let action = SingleAction::Move(rel, piece);
                game.write().apply(start, action);
                GameState::PlacingDuck(start, action)
            } else {
                GameState::Promotion(start, rel, options)
            }
        }
        GameState::PlacingDuck(start, action) => {
            if game.read().valid_duck(loc) {
                game.write().apply_duck(loc);
                let turn = WithId {
                    id,
                    t: TurnRaw {
                        from: start,
                        action,
                        duck_to: loc,
                    },
                };
                game.write().turns.push(turn.t);
                spawn(async move {
                    let json = serde_json::to_string(&turn).unwrap();
                    Request::post("/api/turn").body(json).send().await.unwrap();
                });
                GameState::Waiting
            } else {
                GameState::PlacingDuck(start, action)
            }
        }
    };
    game_state.set(new_game_state);
}
