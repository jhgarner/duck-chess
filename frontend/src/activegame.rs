use std::collections::HashMap;

use common::ChessBoard;
use common::game::GameRaw;
use game::SomeGame;

use crate::{
    board::{Active, DrawBoard, DrawMenuBoard, Drawable, Mods},
    prelude::*,
};

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub enum GameState<Board: ChessBoard> {
    #[default]
    Waiting,
    MyMove,
    Selected(Board::Loc, HashMap<Board::Loc, ActionRaw<Board::Rel>>),
    Promotion(Board::Loc, Board::Rel, Vec<Piece>),
    PlacingDuck(Board::Loc, SingleAction<Board::Rel>),
}
use GameState::*;

enum UIState<Board: ChessBoard> {
    InMenu(Vec<Piece>, Board::Loc, Board::Rel),
    Main(Active<Board::Loc>, HashSet<Board::Loc>),
}

#[component]
pub fn SomeActiveGame(id: ObjectId, game: Game) -> Element {
    let colors = game.player(&use_context());
    match game.some_game {
        SomeGame::Square(og_game) => rsx!(ResetableActiveGame {
            id,
            colors,
            og_game
        }),
        SomeGame::Hex(og_game) => rsx!(ResetableActiveGame {
            id,
            colors,
            og_game
        }),
    }
}

#[component]
fn ResetableActiveGame<Board: Drawable>(
    id: ObjectId,
    colors: PlayerColor,
    og_game: GameRaw<Board>,
) -> Element {
    let game = with_signal(og_game);
    let state = with_signal(MyMove);
    rsx!(ActiveGame {
        id,
        colors,
        game,
        state
    })
}

#[component]
pub fn ActiveGame<Board: Drawable>(
    id: ObjectId,
    colors: PlayerColor,
    game: Signal<GameRaw<Board>>,
    state: Signal<GameState<Board>>,
) -> Element {
    let board = Some::Mapped(game.map(|game| &game.board));

    let ui_state: UIState<Board> = match state() {
        Selected(start, actions) => {
            let targets = actions.keys().copied().collect();
            UIState::Main(Active::Active(start), targets)
        }
        PlacingDuck(_, _) => {
            let targets = game.read().empties();
            let duck = game.read().duck_loc.into();
            UIState::Main(duck, targets)
        }
        MyMove | Waiting => UIState::Main(Active::NoActive, HashSet::new()),
        Promotion(loc, rel, pieces) => UIState::InMenu(pieces, loc, rel),
    };

    match ui_state {
        UIState::Main(active, targets) => rsx! {
            DrawBoard::<Board> {
                action: move |loc| {
                    let updated = update(id, game, state.take(), loc);
                    state.set(updated);
                },
                board,
                mods: Mods::new(active, targets),
                colors,
            }
        },
        UIState::InMenu(pieces, start, rel) => rsx! {
            DrawMenuBoard {
                color: game.read().turn(),
                pieces: pieces,
                action: move |piece| {
                    let action = SingleAction::Move(rel, piece);
                    game.write().apply_from(start, action);
                    state.set(PlacingDuck(start, action));
                },
            }
        },
    }
}

fn update<Board: ChessBoard>(
    id: ObjectId,
    mut game: Signal<GameRaw<Board>>,
    game_state: GameState<Board>,
    loc: Board::Loc,
) -> GameState<Board> {
    match game_state {
        Waiting => Waiting,
        MyMove => {
            let valid_moves = game.read().valid_locations_from(loc);
            if valid_moves.is_empty() {
                MyMove
            } else {
                Selected(loc, valid_moves)
            }
        }
        Selected(start, mut valid_moves) => {
            if let Some(action) = valid_moves.remove(&loc) {
                match action {
                    ActionRaw::Promotion(rel, options) => GameState::Promotion(start, rel, options),
                    ActionRaw::Just(action) => {
                        game.write().apply_from(start, action);
                        PlacingDuck(start, action)
                    }
                }
            } else {
                MyMove
            }
        }
        PlacingDuck(start, action) => {
            if game.read().valid_duck(loc) {
                game.write().apply_duck(loc);
                let turn = WithId::new(
                    id,
                    TurnRaw {
                        from: start,
                        action,
                        duck_to: loc,
                    },
                );
                let some_turn = WithId::new(id, Board::wrap_turn(turn.t));
                game.write().turns.push(turn.t);
                spawn(async move {
                    let json = serde_json::to_string(&some_turn).unwrap();
                    Request::post("/api/turn").body(json).send().await.unwrap();
                });
                Waiting
            } else {
                PlacingDuck(start, action)
            }
        }
        // If the user clicks on the main board while the promotion menu is up,
        // cancel the action
        Promotion(_, _, _) => MyMove,
    }
}
