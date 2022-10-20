use crate::{board::game_preview, prelude::*};

#[inline_props]
pub fn new_game(cx: &Scope) -> Element {
    let open_games = use_future(&cx, || async {
        let response = Request::get("api/open_games").send().await.unwrap();
        response.json::<Vec<WithId<GameRequest>>>().await.unwrap()
    });

    let router = use_router(&cx);
    let mut previews = Vec::new();
    if let Some(games) = open_games.value() {
        for game in games {
            let id = game.id.to_string();
            previews.push(game_preview(router, id, Board::static_default()));
        }
        if previews.is_empty() {
            previews.push(rsx! {
                "No open games, so start your own!"
            });
        }
    }

    cx.render(rsx! {
        div {
            class: "newGame",
            button {
                onclick: move |_| {
                    let router = router.clone();
                    cx.push_future(async move {
                        let response = Request::post("api/new_game").send().await.unwrap();
                        let id = response.json::<ObjectId>().await.unwrap();
                        let to = format!("/ui/game/{id}");
                        router.push_route(&to, None, None);
                    });
                },
                "Create a new game"
            }
            "Or pick a game to join"
            hr {}
            div {
                class: "newGamePreviews",
                previews.into_iter()
            }
        }
    })
}
