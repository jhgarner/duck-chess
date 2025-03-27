use game::GameTypes;
use hexboard::Hexboard;

use crate::{
    board::{DrawBoard, Drawable, Mods},
    prelude::*,
};

#[component]
pub fn JoinableGame(id: ObjectId, request: GameRequest) -> Element {
    let maker = request.maker.name.as_str();
    match request.game_type {
        GameTypes::Square => joinable_board(id, maker, Board::static_default()),
        GameTypes::Hex => joinable_board(id, maker, Hexboard::static_default()),
    }
}

pub fn joinable_board<Board: Drawable>(
    id: ObjectId,
    maker: &str,
    board: &'static Board,
) -> Element {
    rsx! {
        div {
            class: "headed",
            button {
                onclick: move |_| async move {
                        Request::post(&format!("/api/join_game/{}", id)).send().await.unwrap();
                },
                "join \"{maker}\" in a game!"
            }
            DrawBoard::<Board> {
                action: |_| {},
                board,
                mods: Mods::default(),
                colors: PlayerColor::None
            }
        }
    }
}
