mod activegame;
mod board;
mod ingame;
mod joinablegame;
mod loading;
mod loginbuttons;
mod mainmenu;
mod newgame;
mod notification;
mod padding;
mod prelude;
mod route;
mod some;
mod tracked;
mod transition;
mod unauth;
mod use_sse;

use prelude::*;

fn app() -> Element {
    let player_future = use_resource(|| async {
        let response = Request::post("/api/session").send().await.unwrap();
        response.json::<Player>().await.ok()
    });
    if let Some(response) = player_future.value()() {
        if let Some(player) = response {
            use_context_provider(|| player);
            rsx! {
                Router::<route::Route> {}
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
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    launch(app)
}
