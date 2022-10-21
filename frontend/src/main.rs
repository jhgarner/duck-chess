mod activegame;
mod board;
mod ingame;
mod joinablegame;
mod loading;
mod mainmenu;
mod myusefuture;
mod newgame;
mod prelude;
mod unauth;
mod use_event_listener;

use prelude::*;

fn app(cx: Scope) -> Element {
    let player_future = use_future(&cx, || async {
        let response = Request::post("/api/session").send().await.unwrap();
        response.json::<Player>().await.ok()
    });
    let html = if let Some(response) = player_future.value() {
        if let Some(player) = response {
            rsx! {
                main {
                    div {
                        class: "empty",
                    }
                    Router {
                        Route {
                            to: "/",
                            mainmenu::main_menu {
                                player: player
                            }
                        }
                        Route {
                            to: "/ui/newgame",
                            newgame::new_game { }
                        }
                        Route {
                            to: "/ui/game/:id",
                            ingame::in_game {
                                player: player
                            }
                        }
                    }
                    div {
                        class: "empty",
                    }
                }
            }
        } else {
            rsx! {
                unauth::unauth {
                    session: player_future
                }
            }
        }
    } else {
        rsx! { div {}}
    };

    cx.render(html)
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::web::launch(app)
}
