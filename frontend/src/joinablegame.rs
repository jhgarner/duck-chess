use crate::{
    board::{self, Active},
    prelude::*,
};

#[inline_props]
pub fn joinable_game(cx: &Scope, id: ObjectId, request: GameRequest) -> Element {
    let maker = &request.maker.name;

    cx.render(rsx! {
        button {
            onclick: move |_| {
                let id = *id;
                cx.push_future(async move {
                    Request::post(&format!("/api/join_game/{}", id)).send().await.unwrap();
                });
            },
            "join \"{maker}\" in a game!"
        }
        div {
            class: "content",
            board::board {
                action: |_| {},
                board: Cow::Borrowed(Board::static_default()),
                active: Active::NoActive,
                targets: HashSet::new()
            }
        }
    })
}
