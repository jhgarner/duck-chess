use web_sys::window;

use crate::board::{game_preview, some_game_preview};
use crate::{notification, prelude::*};

#[component]
pub fn MainMenu() -> Element {
    let player: Player = use_context();

    let mut my_turn = Vec::new();
    let mut other_turn = Vec::new();
    let mut completed = Vec::new();
    let mut open = Vec::new();

    for any_game in use_all_games().iter() {
        let id = any_game.read().id.unwrap().to_string();
        match &any_game.read().game {
            GameOrRequest::Game(game) if game.is_player_turn(&player) => {
                my_turn.push(some_game_preview(id, game))
            }
            GameOrRequest::Game(game) => other_turn.push(some_game_preview(id, game)),
            GameOrRequest::Completed(game) => completed.push(some_game_preview(id, game)),
            GameOrRequest::Request(_) => open.push(game_preview::<Board>(
                id,
                PlayerColor::None,
                Board::static_default(),
            )),
        }
    }

    rsx! {
        div {
            class: "mainMenu",
            div {
                class: "header",
                h1 {
                    "Duck Chess"
                }
                div {
                    class: "buttonMenu",
                    notification::subscribe {}
                    button {
                        onclick: move |_| async {
                            crate::rpc::logout().await.unwrap();
                            window().unwrap().location().reload().unwrap();
                        },
                        "Logout all devices"
                    }
                    Link { to: "/ui/newgame", "New Game" }
                }
            }
            h2 { "Your Turn" },
            {my_turn.into_iter()},
            h2 { "Their Turn" },
            {other_turn.into_iter()},
            h2 { "Open games" },
            {open.into_iter()},
            h2 { "Completed Games" },
            {completed.into_iter()}
        }
    }
}
