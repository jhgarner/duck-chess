use hexboard::Coord;

use super::*;

#[component]
pub fn DrawBlock<Loc: Gridable>(square: Square, at: Loc, action: EventHandler<Loc>) -> Element {
    let src = format!("/assets/{}.svg", square.name());
    let Block { x, y, w, h } = reversable_grid(at);

    let colors = Loc::row_colors();
    let row_colors = colors[at.into().y % colors.len()];
    let background_style = row_colors[at.into().x % row_colors.len()];

    let row = format!("grid-row: {y} / span {h};");
    let column = format!("grid-column: {x} / span {w};");

    rsx! {
        div {
            class: "overlapper",
            style: "{row}{column}",
            div {
                class: "overlapper background",
                onclick: move |_| action(at),
                div { class: "{background_style}" }
                ActiveHighlight { at }
            }
            WithTranslation {
                at, square, src
            }
            TargetHighlight { at }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl From<Loc> for Block {
    fn from(value: Loc) -> Self {
        Block {
            x: value.right + 1,
            y: value.down + 1,
            w: 1,
            h: 1,
        }
    }
}

impl From<Coord> for Block {
    fn from(coord: Coord) -> Self {
        let radius = 5;
        let col = coord.q;
        let row = 2 * coord.r + coord.q;
        let x = (col + radius) * 3 + 1;
        let y = (row + radius * 2) + 1;
        Block {
            x: x as usize,
            y: y as usize,
            w: 4,
            h: 2,
        }
    }
}

pub trait Gridable: Into<Block> + 'static + Copy + PartialEq + Eq + Hash {
    fn height() -> usize;
    fn row_colors() -> ColorGrid;
}

static EVEN: &str = "cellEven";
static ODD: &str = "cellOdd";
static EXTRA_ODD: &str = "cellExtraOdd";

impl Gridable for Loc {
    fn height() -> usize {
        9
    }

    fn row_colors() -> ColorGrid {
        static ROWS: ColorGrid = &[&[EVEN, ODD], &[ODD, EVEN]];
        ROWS
    }
}

impl Gridable for Coord {
    fn height() -> usize {
        22
    }

    fn row_colors() -> ColorGrid {
        static ROWS: ColorGrid = &[&[EVEN], &[ODD], &[EXTRA_ODD]];
        ROWS
    }
}

type ColorGrid = &'static [&'static [&'static str]];
