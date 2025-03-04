use crate::{board::game_preview, prelude::*, route::Route};

#[component]
pub fn NewGame() -> Element {
    let open_games = use_resource(|| async {
        let response = Request::get("api/open_games").send().await.unwrap();
        response.json::<Vec<WithId<GameRequest>>>().await.unwrap()
    });

    let mut previews = Vec::new();
    if let Some(games) = open_games.value()() {
        for game in games {
            let id = game.id.to_string();
            previews.push(game_preview(id, Board::static_default().clone()));
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
                        let response = Request::post("api/new_game").send().await.unwrap();
                        let id = response.json::<ObjectId>().await.unwrap().to_string();
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
