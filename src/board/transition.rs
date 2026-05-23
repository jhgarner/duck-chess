use super::*;

#[component]
pub fn WithTranslation(block: Block, square: Square, src: String) -> Element {
    let BoardId { hero, .. } = use_context();
    match square {
        Square::Piece(_, _, SquareId(id)) => {
            let transition_name = if hero {
                format!("_{id}")
            } else {
                "".to_string()
            };
            rsx!(Keyed {
                img {
                    key: "{src}",
                    id: transition_name,
                    style: "z-index: 100;",
                    src
                }
            })
        }
        Square::Empty => {
            rsx!(Keyed {
                div {
                    style: "z-index: 100;",
                }
            })
        }
        Square::Duck => {
            rsx!(Keyed {
                img {
                    id: "duck",
                    style: "z-index: 100;",
                    src
                }
            })
        }
    }
}
