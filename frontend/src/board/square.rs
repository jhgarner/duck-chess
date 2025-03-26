use modifiers::{ActiveHighlight, TargetHighlight};

use super::*;

pub fn draw(board: Some<Board>, action: EventHandler<Loc>) -> Element {
    let mut board_html: Vec<Element> = Vec::new();
    for (down, row) in board.read().rows().enumerate() {
        for (right, square) in row.iter().enumerate() {
            let at = Loc::new(right, down);
            board_html.push(rsx!(DrawSquare {
                at,
                square: *square,
                action: action,
            }));
        }
    }

    let width = board.read().width();
    let height = board.read().height();
    let columns = format!("grid-template-columns: repeat({width}, 1fr);");
    let aspect_ratio = format!("aspect-ratio: {width}/{height};");

    rsx!(
        Padded {
            padding: Padding::all(8),
            div {
                class: "board",
                style: "{columns}{aspect_ratio}",
                {board_html.into_iter()}
            }
        }
    )
}

#[component]
pub fn DrawSquare(square: Square, at: Loc, action: EventHandler<Loc>) -> Element {
    let class = if (at.down + at.right) % 2 == 0 {
        "cellEven"
    } else {
        "cellOdd"
    };

    let src = format!("/assets/{}.svg", square.name());

    rsx!(
        div {
            class: "overlapper {class}",
            onclick: move |_| action(at),
            ActiveHighlight { at }
            WithTranslation {
                at, square,
                img { src }
            }
            TargetHighlight { at }
        }
    )
}
