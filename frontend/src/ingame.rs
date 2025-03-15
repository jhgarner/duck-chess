use game::{GameRaw, SomeGame};

use crate::board::{Active, DrawBoard};
use crate::{activegame, joinablegame};
use crate::{notification, prelude::*};

#[component]
pub fn InGame(id: String) -> Element {
    let player: Player = use_context();
    let game_or_request = use_sse::<Option<AnyGame>>(format!("/api/poll/{id}"));

    match game_or_request.read().clone() {
        None => rsx! {
            div {
                class: "turnHeaderDiv",
                span {
                    class: "turnHeader",
                    ""
                }
                notification::subscribe {}
            }
            activegame::active_game {
                id: ObjectId::default(),
                og_game: GameRaw::empty_board(),
            }
        },
        Some(None) => rsx! { "Invalid game id" },
        Some(Some(with_id)) => match with_id.game {
            GameOrRequest::Request(request) => {
                rsx! {
                    if request.maker.id == player.id {
                        "Your game hasn't started yet. Share this page to invite someone."
                    } else {
                        ""
                    }
                    joinablegame::joinable_game {
                        request: request,
                        id: with_id.id.unwrap(),
                    }
                }
            }
            GameOrRequest::Game(SomeGame::Square(game))
            | GameOrRequest::Completed(SomeGame::Square(game)) => {
                match get_game_state(&game, &player) {
                    TurnState::MyTurn => rsx! {
                        div {
                            class: "turnHeaderDiv",
                            span {
                                class: "turnHeader",
                                "It is your turn!"
                            }
                            notification::subscribe {}
                        }
                        activegame::active_game {
                            id: with_id.id.unwrap(),
                            og_game: game,
                        }
                    },
                    TurnState::OtherTurn => rsx! {
                        div {
                            class: "turnHeaderDiv",
                            span {
                                class: "turnHeader",
                                "It is not your turn"
                            }
                            notification::subscribe {}
                        }
                        activegame::active_game {
                            id: with_id.id.unwrap(),
                            og_game: game,
                        }
                    },
                    TurnState::Ended(winner) => rsx! {
                        "{winner:?} Won!"
                        DrawBoard {
                            action: |_| {},
                            active: Active::NoActive,
                            board: game.board,
                            targets: HashSet::new()
                        }
                    },
                }
            }
        },
    }
}

enum TurnState {
    MyTurn,
    OtherTurn,
    Ended(Color),
}

fn get_game_state(game: &Game, player: &Player) -> TurnState {
    if let Some(color) = game.game_over() {
        TurnState::Ended(color)
    } else if game.player(player).contains(&game.turn()) {
        TurnState::MyTurn
    } else {
        TurnState::OtherTurn
    }
}
