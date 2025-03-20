use hexboard::Coord;

use super::*;

pub fn draw(
    board: Some<Hexboard>,
    action: EventHandler<Coord>,
    active: Active<Coord>,
    targets: HashSet<Coord>,
) -> Element {
    let mut board_html: Vec<Element> = Vec::new();
    for (at, square) in board.read().iter() {
        board_html.push(rsx!(DrawHex {
            at,
            square: square,
            action: action,
            is_active: Active::Active(at) == active,
            is_target: targets.contains(&at),
        }));
    }

    let diameter = 11;
    let columns = format!("grid-template-columns: repeat({diameter}, 1fr 2fr) 1fr;");
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
pub fn DrawHex(
    square: Square,
    at: Coord,
    is_active: bool,
    is_target: bool,
    action: EventHandler<Coord>,
) -> Element {
    let mut classes = "".to_string();

    let piece = format!("/assets/{}.svg", square.name());

    let active_overlay = if is_active {
        rsx! {
            div {
                class: "active",
            }
        }
    } else {
        rsx! {}
    };

    let target_overlay = if is_target {
        rsx! {
            div {
                class: "target",
            }
        }
    } else {
        rsx! {}
    };

    if is_target {
        classes.push_str(" target");
    }

    let (x, y) = to_grid(at);

    let mod3 = y % 3;
    if mod3 == 0 {
        classes.push_str(" cellEven");
    } else if mod3 == 1 {
        classes.push_str(" cellOdd");
    } else if mod3 == 2 {
        classes.push_str(" cellExtraOdd");
    }

    let row = format!("grid-row: {y} / span 2;");
    let column = format!("grid-column: {x} / span 3;");

    rsx! {
        div {
            class: "item-content {classes}",
            style: "{row}{column}",
            onclick: move |_| action(at),
            {active_overlay}
            {target_overlay}
            img {
                src: "{piece}"
            }
        }
    }
}

fn to_grid(coord: Coord) -> (i32, i32) {
    let radius = 5;
    let col = coord.q;
    let row = 2 * coord.r + coord.q;
    ((col + radius) * 2 + 1, (row + radius * 2) + 1)
}
