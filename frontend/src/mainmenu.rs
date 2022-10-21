use web_sys::window;

use crate::board::game_preview;
use crate::{notification, prelude::*};

#[inline_props]
pub fn main_menu<'a>(cx: Scope<'a>, player: &'a Player) -> Element {
    let my_games = use_future(&cx, || async {
        let response = Request::get("api/games").send().await.unwrap();
        response.json::<Vec<AnyGame>>().await.unwrap()
    });

    if let Some(my_games) = my_games.value() {
        let mut my_turn = Vec::new();
        let mut other_turn = Vec::new();
        let mut completed = Vec::new();
        let mut open = Vec::new();

        let router = use_router(&cx);

        for any_game in my_games {
            let id = any_game.id.unwrap().to_string();
            match &any_game.game {
                GameOrRequest::Game(game) if game.is_player_turn(player) => {
                    my_turn.push(game_preview(router, id, &game.board))
                }
                GameOrRequest::Game(game) => other_turn.push(game_preview(router, id, &game.board)),
                GameOrRequest::Completed(game) => {
                    completed.push(game_preview(router, id, &game.board))
                }
                GameOrRequest::Request(_) => {
                    open.push(game_preview(router, id, Board::static_default()))
                }
            }
        }

        cx.render(rsx!(div {
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
                        onclick: move |_| {cx.push_future(async {
                            Request::post("/api/logout").send().await.unwrap();
                            window().unwrap().location().reload().unwrap();
                        });},
                        "Logout all devices"
                    }
                    Link { to: "/ui/newgame", "New Game" }
                }
            }
            h2 { "Your Turn" },
            my_turn,
            h2 { "Their Turn" },
            other_turn,
            h2 { "Open games" },
            open,
            h2 { "Completed Games" },
            completed
        }))
    } else {
        cx.render(spinner())
    }
}
