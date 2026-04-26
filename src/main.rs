mod activegame;
mod board;
mod common;
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
mod tracked;
mod unauth;
mod use_sse;

use prelude::*;

fn app() -> Element {
    let player_future = use_resource(|| async { rpc::fetch_session().await.ok().flatten() });
    if let Some(response) = player_future.value()() {
        if let Some(player) = response {
            use_context_provider(|| player);
            rsx! {
                document::Stylesheet { href: asset!("index.css") }
                Router::<route::Route> {}
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
