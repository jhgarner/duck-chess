use crate::prelude::*;

#[derive(PartialEq, Eq)]
pub enum Active<Loc> {
    Active(Loc),
    NoActive,
}

impl<Loc> From<Option<Loc>> for Active<Loc> {
    fn from(opt: Option<Loc>) -> Self {
        opt.map_or(Active::NoActive, Active::Active)
    }
}

#[derive(Props)]
pub struct Propss<'a, T, Loc> {
    pub action: T,
    pub board: Cow<'a, Board>,
    pub active: Active<Loc>,
    pub targets: HashSet<Loc>,
}

// If inline_props supported where clauses, this would work...
// #[inline_props]
pub fn board<'a, T: Fn(Loc) + 'a, Loc>(cx: Scope<'a, Propss<'a, T, Loc>>) -> Element {
    let Propss {
        action,
        board,
        active,
        targets,
    } = &cx.props;
    let mut board_html: Vec<LazyNodes> = Vec::new();
    for (down, row) in board.rows().enumerate() {
        for (right, square) in row.iter().enumerate() {
            let at = Loc::new(right, down);

            let mut classes = "square".to_string();

            if (down + right) % 2 == 0 {
                classes.push_str(" cellEven");
            } else {
                classes.push_str(" cellOdd");
            };

            let piece = format!("/assets/{}.svg", square.name());

            if let Active::Active(loc) = *active {
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

    let width = board.width();
    let style = format!("grid-template-columns: repeat({width}, 1fr);");

    cx.render(rsx!(
        div {
            class: "boardHolder",
            div {
                class: "board",
                style: "{style}",
                board_html.into_iter()
            }
        }
    ))
}

pub fn game_preview<'a>(
    router: &'a RouterService,
    id: String,
    board: &'a Board,
) -> LazyNodes<'a, 'a> {
    let to = format!("/ui/game/{id}");
    let cloned_to = to.clone();
    rsx!(div {
        style: "width: 200px; height: 200px",
        a {
            href: "{to}",
            onclick: move |_| {
                router.push_route(&cloned_to, None, None);
            },
            self::board {
                action: &|_| {},
                board: Cow::Borrowed(board),
                active: Active::NoActive,
                targets: HashSet::new(),
            }
        }
    })
}
