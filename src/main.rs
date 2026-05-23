mod activegame;
mod board;
mod common;
mod global;
mod ingame;
mod joinablegame;
mod keyed;
mod loading;
mod loginbuttons;
mod mainmenu;
mod newgame;
mod notification;
mod padding;
mod prelude;
mod route;
mod rpc;
#[cfg(feature = "server")]
mod server;
mod some;
mod style;
mod tracked;
mod transition;
mod unauth;
mod update_gate;
mod use_sse;

use std::sync::Arc;

use prelude::*;

use crate::{
    style::GlobalStyle,
    transition::transition,
    update_gate::{Gate, close_gate, open_gate},
};

fn app() -> Element {
    let player_future = use_resource(|| async { rpc::fetch_session().await.ok().flatten() });
    use_effect(|| {
        let history = history();
        history.updater(Arc::new(|| {
            log::warn!("In the updater callback");
            close_gate();
            transition(|resolver| {
                log::warn!("About to open the gate");
                open_gate();
                log::warn!("Opened the gate");
                resolver.finish();
            });
        }));
    });
    if let Some(response) = player_future.value()() {
        if let Some(player) = response {
            use_context_provider(|| player);
            rsx! {
                document::Stylesheet { href: asset!("index.css") }
                GlobalStyle {
                    Gate {
                        Router::<route::Route> {}
                    }
                }
            }
        } else {
            rsx! {
                document::Stylesheet { href: asset!("index.css") }
                unauth::unauth {
                    session: player_future
                }
            }
        }
    } else {
        rsx! { div {}}
    }
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "web")]
    serve_web();

    #[cfg(feature = "server")]
    serve_server();

    Ok(())
}

#[cfg(feature = "web")]
fn serve_web() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::launch(app);
}

#[cfg(feature = "server")]
fn serve_server() {
    dioxus::serve(|| async move {
        let router = server::build_state(dioxus::server::router(app))
            .await?
            .layer(tower_cookies::CookieManagerLayer::new());

        Ok(router)
    });
}
