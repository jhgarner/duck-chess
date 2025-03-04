use common::board::ChessBoard;

use crate::{prelude::*, route::Route};

pub trait DrawableBoard: ChessBoard {
    fn draw(
        &self,
        action: EventHandler<Self::Loc>,
        active: Active<Self::Loc>,
        targets: &HashSet<Self::Loc>,
    ) -> Element;
}

impl DrawableBoard for Board {
    fn draw(
        &self,
        action: EventHandler<Self::Loc>,
        active: Active<Self::Loc>,
        targets: &HashSet<Self::Loc>,
    ) -> Element {
        let mut board_html: Vec<Element> = Vec::new();
        for (down, row) in self.rows().enumerate() {
            for (right, square) in row.iter().enumerate() {
                let at = Loc::new(right, down);

                let mut classes = "square".to_string();

                if (down + right) % 2 == 0 {
                    classes.push_str(" cellEven");
                } else {
                    classes.push_str(" cellOdd");
                };

                let piece = format!("/assets/{}.svg", square.name());

                if let Active::Active(loc) = active {
                    if loc == at {
                        classes.push_str(" active");
                    }
                }
                if targets.contains(&at) {
                    classes.push_str(" target");
                }

                board_html.push(rsx!(
                    div {
                        class: "{classes}",
                        onclick: move |_| action(Loc::new(right, down)),
                        img {
                            src: "{piece}"
                        }
                    }
                ));
            }
        }

        let width = self.width();
        let height = self.height();
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

#[component]
pub fn BoardC(
    action: EventHandler<Loc>,
    board: Board,
    active: Active<Loc>,
    targets: HashSet<Loc>,
) -> Element {
    board.draw(action, active, &targets)
}

pub fn game_preview(id: String, board: Board) -> Element {
    rsx!(div {
        style: "width: 200px; height: 200px",
        Link {
            to: Route::InGame {id},
            self::BoardC {
                action: &|_| {},
                board: board,
                active: Active::NoActive,
                targets: HashSet::new(),
            }
        }
    })
}
