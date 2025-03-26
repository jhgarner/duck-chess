use hexboard::Coord;

use super::*;

pub fn draw(board: Some<Hexboard>, action: EventHandler<Coord>) -> Element {
    let mut board_html: Vec<Element> = Vec::new();
    for (at, square) in board.read().iter() {
        board_html.push(rsx!(DrawHex {
            at,
            square: square,
            action: action,
        }));
    }

    let diameter = 11;
    let columns = format!("grid-template-columns: repeat({diameter}, 1fr 1fr 1fr) 1fr;");
    let rows = format!("grid-template-rows: repeat({diameter}, 1fr 1fr);");

    rsx!(
        Padded {
            padding: Padding::all(8),
            div {
                class: "hexboard",
                style: "{columns}{rows}",
                {board_html.into_iter()}
            }
        }
    )
}

#[component]
pub fn DrawHex(square: Square, at: Coord, action: EventHandler<Coord>) -> Element {
    let mut classes = "".to_string();

    let piece = format!("/assets/{}.svg", square.name());

    let Grid { x, y, .. } = at.into();

    let mod3 = y % 3;
    if mod3 == 0 {
        classes.push_str(" cellEven");
    } else if mod3 == 1 {
        classes.push_str(" cellOdd");
    } else if mod3 == 2 {
        classes.push_str(" cellExtraOdd");
    }

    let row = format!("grid-row: {y} / span 2;");
    let column = format!("grid-column: {x} / span 4;");

    rsx! {
        div {
            class: "overlapper",
            style: "{row}{column}",
            onclick: move |_| action(at),
            div {
                class: "overlapper background",
                div { class: "{classes}" }
                ActiveHighlight { at }
                TargetHighlight { at }
            }
            WithTranslation {
                at, square,
                img {
                    src: "{piece}"
                }
            }
        }
    }
}
