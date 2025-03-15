use menuboard::MenuBoard;

use crate::{prelude::*, route::Route};

#[component]
pub fn DrawMenuBoard(color: Color, pieces: Vec<Piece>, action: EventHandler<Piece>) -> Element {
    let squares = pieces
        .iter()
        .map(|piece| Square::Piece(color, *piece))
        .collect();
    rsx! {
        DrawBoard {
            board: Board::from(MenuBoard::new(squares)),
            action: move |loc: Loc| {action(pieces[loc.right])},
            active: Active::NoActive,
            targets: HashSet::new(),
        }
    }
}

#[component]
pub fn DrawSquare(
    square: Square,
    at: Loc,
    is_active: bool,
    is_target: bool,
    action: EventHandler<Loc>,
) -> Element {
    let mut classes = "square".to_string();

    if (at.down + at.right) % 2 == 0 {
        classes.push_str(" cellEven");
    } else {
        classes.push_str(" cellOdd");
    };

    let piece = format!("/assets/{}.svg", square.name());

    if is_active {
        classes.push_str(" active");
    }
    if is_target {
        classes.push_str(" target");
    }

    rsx!(
        div {
            class: "{classes}",
            onclick: move |_| action(at),
            img {
                src: "{piece}"
            }
        }
    )
}

#[component]
pub fn DrawBoard(
    #[props(into)] board: Some<Board>,
    action: EventHandler<Loc>,
    active: Active<Loc>,
    targets: HashSet<Loc>,
) -> Element {
    let mut board_html: Vec<Element> = Vec::new();
    for (down, row) in board.read().rows().enumerate() {
        for (right, square) in row.iter().enumerate() {
            let at = Loc::new(right, down);
            board_html.push(rsx!(DrawSquare {
                at,
                square: *square,
                action: action,
                is_active: Active::Active(at) == active,
                is_target: targets.contains(&at),
            }));
        }
    }

    let width = board.read().width();
    let height = board.read().height();
    let columns = format!("grid-template-columns: repeat({width}, 1fr);");
    let aspect_ratio = format!("aspect-ratio: {width}/{height};");

    rsx!(
        div {
            class: "boardHolder",
            div {
                class: "board",
                style: "{columns}{aspect_ratio}",
                {board_html.into_iter()}
            }
        }
    )
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Active<Loc> {
    Active(Loc),
    NoActive,
}

impl<Loc> From<Option<Loc>> for Active<Loc> {
    fn from(opt: Option<Loc>) -> Self {
        opt.map_or(Active::NoActive, Active::Active)
    }
}

pub fn game_preview(id: String, board: Board) -> Element {
    rsx!(div {
        style: "width: 200px; height: 200px",
        Link {
            to: Route::InGame {id},
            DrawBoard {
                action: &|_| {},
                board: board,
                active: Active::NoActive,
                targets: HashSet::new(),
            }
        }
    })
}
