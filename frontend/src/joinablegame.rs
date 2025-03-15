use crate::{
    board::{self, Active},
    prelude::*,
};

#[component]
pub fn joinable_game(id: ObjectId, request: GameRequest) -> Element {
    let maker = request.maker.name;

    rsx! {
        button {
            onclick: move |_| async move {
                    Request::post(&format!("/api/join_game/{}", id)).send().await.unwrap();
            },
            "join \"{maker}\" in a game!"
        }
        div {
            class: "content",
            board::DrawBoard {
                action: |_| {},
                board: Board::static_default(),
                active: Active::NoActive,
                targets: HashSet::new()
            }
        }
    }
}
