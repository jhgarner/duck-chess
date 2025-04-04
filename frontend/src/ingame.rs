use std::ops::Deref;

use crate::activegame::SomeActiveGame;
use crate::board::DrawSomeGame;
use crate::joinablegame::JoinableGame;
use crate::{notification, prelude::*};

#[derive(Clone, PartialEq)]
pub enum ServerTurn {
    Loading,
    Invalid,
    NotStarted(ObjectId, GameRequest),
    MyTurn(ObjectId, Game),
    OtherTurn(Game),
    Ended(Color, Game),
}

#[component]
pub fn InGame(id: String) -> Element {
    let player: Player = use_context();
    let game_or_request = use_sse::<Option<AnyGame>>(format!("/api/poll/{id}"));
    let mut server_turn = use_signal(|| ServerTurn::Loading);

    *server_turn.write() = match game_or_request.read().clone() {
        None => ServerTurn::Loading,
        Some(None) => ServerTurn::Invalid,
        Some(Some(with_id)) => match with_id.game {
            GameOrRequest::Request(request) => ServerTurn::NotStarted(with_id.id.unwrap(), request),
            GameOrRequest::Game(game) | GameOrRequest::Completed(game) => {
                let state = get_game_state(&game, &player);
                match state {
                    TurnState::MyTurn => ServerTurn::MyTurn(with_id.id.unwrap(), game),
                    TurnState::OtherTurn => ServerTurn::OtherTurn(game),
                    TurnState::Ended(winner) => ServerTurn::Ended(winner, game),
                }
            }
        },
    };

    match server_turn.read().deref().clone() {
        ServerTurn::Loading => spinner(),
        ServerTurn::Invalid => rsx! { "Invalid game id" },
        ServerTurn::NotStarted(id, request) => rsx! {
            div {
                class: "headed",
                if request.maker.id == player.id {
                    "Your game hasn't started yet. Share this page to invite someone."
                } else {
                    div {}
                }
                JoinableGame {
                    request,
                    id,
                }
            }
        },
        ServerTurn::MyTurn(id, game) => rsx! {
            div {
                class: "headed",
                div {
                    class: "turnHeaderDiv",
                    span {
                        class: "turnHeader",
                        "It is your turn!"
                    }
                    notification::subscribe {}
                }
                SomeActiveGame {
                    id,
                    game,
                }
            }
        },
        ServerTurn::OtherTurn(game) => rsx! {
            div {
                class: "headed",
                div {
                    class: "turnHeaderDiv",
                    span {
                        class: "turnHeader",
                        "It is not your turn"
                    }
                    notification::subscribe {}
                }
                DrawSomeGame {
                    game,
                }
            }
        },
        ServerTurn::Ended(winner, game) => rsx! {
            div {
                class: "headed",
                "{winner:?} Won!"
                DrawSomeGame {
                    game,
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
