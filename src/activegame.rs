use std::collections::HashMap;

use crate::common::game::GameRaw;
use crate::style::clear_style;
use crate::transition::transition_callback;
use crate::{common::ChessBoard, style::set_style};
use game::SomeGame;

use crate::{
    board::{Active, DrawBoard, DrawMenuBoard, Drawable, Mods, Select, TargetType, Targetting},
    prelude::*,
};

#[derive_where(Default)]
#[derive(PartialEq, Clone, Debug)]
pub enum GameState<Board: ChessBoard> {
    Waiting,
    #[derive_where(default)]
    MyMove(Option<Targetting<Board::Loc>>),
    Selected(Board::Loc, HashMap<Board::Loc, ActionRaw<Board::Rel>>),
    Promotion(Board::Loc, Board::Rel, Vec<Piece>),
    PlacingDuck(Board::Loc, SingleAction<Board::Rel>),
}
use GameState::*;

enum UIState<Board: ChessBoard> {
    InMenu(Vec<Piece>, Board::Loc, Board::Rel),
    Main(Targetting<Board::Loc>),
}

impl<Board: ChessBoard> UIState<Board> {
    pub fn main(active: Active<Board::Loc>, targets: HashSet<Board::Loc>) -> Self {
        UIState::Main(Targetting::pick(active, targets))
    }
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
    let state = with_signal(GameState::default());
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
    let board: Some<Board> = game.map(|game| &game.board).into();
    let dangers = Color::all()
        .into_iter()
        .filter_map(|c| game.read().checked(c))
        .collect();

    let ui_state: UIState<Board> = match state() {
        Selected(start, actions) => {
            let targets = actions.keys().copied().collect();
            UIState::main(Active::Active(start), targets)
        }
        PlacingDuck(_, _) => {
            let targets = game.read().empties();
            let duck = game.read().duck_loc.into();
            UIState::main(duck, targets)
        }
        MyMove(None) | Waiting => UIState::main(Active::NoActive, HashSet::new()),
        MyMove(Some(target)) => UIState::Main(target),
        Promotion(loc, rel, pieces) => UIState::InMenu(pieces, loc, rel),
    };

    match ui_state {
        UIState::Main(targetting) => rsx! {
            DrawBoard::<Board> {
                action: move |loc| {
                    let was_selecting = matches!(*state.read(), GameState::Selected(_, _));
                    let updated = select(id, game, state.take(), loc);
                    let now_ducking = matches!(updated, GameState::PlacingDuck(_, _));
                    if was_selecting && now_ducking {
                        // transition_callback(move || {
                            state.set(updated);
                        // });
                    } else {
                        state.set(updated);
                    }
                },
                board,
                mods: Mods::new(vec![targetting], dangers),
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

fn select<Board: ChessBoard>(
    id: ObjectId,
    game: Signal<GameRaw<Board>>,
    game_state: GameState<Board>,
    select: Select<Board::Loc>,
) -> GameState<Board> {
    match select {
        Select::Pick(loc) => update(id, game, game_state, loc),
        Select::Consider(loc) => hover(game, game_state, loc),
        Select::Unconsider => {
            if let MyMove(_) = game_state {
                MyMove(None)
            } else {
                game_state
            }
        }
    }
}

fn hover<Board: ChessBoard>(
    game: Signal<GameRaw<Board>>,
    game_state: GameState<Board>,
    loc: Board::Loc,
) -> GameState<Board> {
    match game_state {
        MyMove(_) => {
            if let Some(Square::Piece(player, _, _)) = game.read().get(loc) {
                let valid_moves = game.read().valid_locations_from_player(loc, player);
                let targets = valid_moves.keys().copied().collect();
                let target_type = if player == game.read().turn() {
                    TargetType::Consider
                } else {
                    TargetType::Info
                };
                MyMove(Some(Targetting::new(
                    Active::Active(loc),
                    targets,
                    target_type,
                )))
            } else {
                MyMove(None)
            }
        }
        this => this,
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
        MyMove(targetting) => {
            let valid_moves = game.read().valid_locations_from(loc);
            if valid_moves.is_empty() {
                MyMove(targetting)
            } else {
                Selected(loc, valid_moves)
            }
        }
        Selected(start, mut valid_moves) => {
            if let Some(action) = valid_moves.remove(&loc) {
                match action {
                    ActionRaw::Promotion(rel, options) => GameState::Promotion(start, rel, options),
                    ActionRaw::Just(action) => {
                        let id = match game.read().get(start).unwrap() {
                            Square::Piece(_, _, SquareId(id)) => format!("_{id}"),
                            Square::Duck => "duck".to_string(),
                            Square::Empty => panic!("Empty squares cannot be moved"),
                        };
                        set_style(
                            "moving_piece",
                            format!("#{id} {{ view-transition-name: moving_piece }}"),
                        );
                        transition_callback(move || {
                            game.write().apply_from(start, action);
                        })
                        .on_complete_callback(|| {
                            clear_style("moving_piece");
                        });
                        PlacingDuck(start, action)
                    }
                }
            } else {
                MyMove(None)
            }
        }
        PlacingDuck(start, action) => {
            if game.read().valid_duck(loc) {
                let turn = WithId::new(
                    id,
                    TurnRaw {
                        from: start,
                        action,
                        duck_to: loc,
                    },
                );
                set_style(
                    "moving_piece",
                    "#duck { view-transition-name: moving_piece }".to_string(),
                );
                transition_callback(move || {
                    game.write().apply_duck(loc);
                    let some_turn = WithId::new(id, Board::wrap_turn(turn.t));
                    game.write().turns.push(turn.t);
                    spawn(async move {
                        crate::rpc::submit_turn_rpc(some_turn).await.unwrap();
                    });
                })
                .on_complete_callback(|| {
                    clear_style("moving_piece");
                });
                Waiting
            } else {
                PlacingDuck(start, action)
            }
        }
        // If the user clicks on the main board while the promotion menu is up,
        // cancel the action
        Promotion(_, _, _) => MyMove(None),
    }
}
