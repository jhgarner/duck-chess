use crate::{board::game_preview, prelude::*, route::Route};

#[component]
pub fn NewGame() -> Element {
    let open_games =
        use_resource(|| async { crate::rpc::fetch_open_games().await.unwrap_or_default() });

    let mut previews = Vec::new();
    if let Some(games) = open_games.value()() {
        for game in games {
            let id = game.id.to_string();
            previews.push(game_preview::<Board>(
                id,
                PlayerColor::None,
                Board::static_default(),
            ));
        }
        if previews.is_empty() {
            previews.push(rsx! {
                "No open games, so start your own!"
            });
        }
    }

    rsx! {
        div {
            class: "newGame",
            button {
                onclick: move |_| async move {
                        let id = crate::rpc::create_game().await.unwrap().to_string();
                        navigator().push(Route::InGame {id});
                },
                "Create a new game"
            }
            "Or pick a game to join"
            hr {}
            div {
                class: "newGamePreviews",
                {previews.into_iter()}
            }
        }
    }
}
