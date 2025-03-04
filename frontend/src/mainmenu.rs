use game::SomeGame;
use web_sys::window;

use crate::board::game_preview;
use crate::{notification, prelude::*};

#[component]
pub fn MainMenu() -> Element {
    let player: Player = use_context();
    let my_games = use_resource(|| async {
        let response = Request::get("api/games").send().await.unwrap();
        response.json::<Vec<AnyGame>>().await.unwrap()
    });

    if let Some(my_games) = my_games.value()() {
        let mut my_turn = Vec::new();
        let mut other_turn = Vec::new();
        let mut completed = Vec::new();
        let mut open = Vec::new();

        for any_game in my_games {
            let id = any_game.id.unwrap().to_string();
            match &any_game.game {
                GameOrRequest::Game(SomeGame::Square(game)) if game.is_player_turn(&player) => {
                    my_turn.push(game_preview(id, game.board.clone()))
                }
                GameOrRequest::Game(SomeGame::Square(game)) => {
                    other_turn.push(game_preview(id, game.board.clone()))
                }
                GameOrRequest::Completed(SomeGame::Square(game)) => {
                    completed.push(game_preview(id, game.board.clone()))
                }
                GameOrRequest::Request(_) => {
                    open.push(game_preview(id, Board::static_default().clone()))
                }
            }
        }
        log::warn!("got my turn {my_turn:?}");
        log::warn!("got not my turn {other_turn:?}");

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
                                Request::post("/api/logout").send().await.unwrap();
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
    } else {
        spinner()
    }
}
