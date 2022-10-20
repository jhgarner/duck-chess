use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Notification, PushSubscriptionOptionsInit, ServiceWorkerRegistration};

use crate::board::game_preview;
use crate::prelude::*;

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
                    button {
                        onclick: move |_| {cx.push_future(subscribe_me());},
                        "Subscribe to Notifications"
                    }
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

// TODO This low level javascript stuff doesn't belong here. Also it's pretty gross
async fn subscribe_me() {
    // Most of this method interacts with the browser API for receiving notifications
    JsFuture::from(Notification::request_permission().unwrap())
        .await
        .unwrap();
    let registration = JsFuture::from(
        window()
            .unwrap()
            .navigator()
            .service_worker()
            .register("assets/worker.js"),
    )
    .await
    .unwrap();
    let key_encoded = Request::get("api/public_key")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut options = PushSubscriptionOptionsInit::new();
    options.application_server_key(Some(&JsValue::from_str(&key_encoded)));
    options.user_visible_only(true);
    let registration = registration
        .dyn_ref::<ServiceWorkerRegistration>()
        .unwrap()
        .push_manager()
        .unwrap();
    let result = JsFuture::from(registration.subscribe_with_options(&options).unwrap())
        .await
        .unwrap();
    Request::post("api/subscribe")
        .body(js_sys::JSON::stringify(&result).unwrap().as_string())
        .send()
        .await
        .unwrap();
}
