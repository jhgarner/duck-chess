use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlFormElement};

use super::*;

#[component]
pub fn DrawBlock<Loc: Gridable>(
    square: Square,
    at: Loc,
    action: EventHandler<Select<Loc>>,
) -> Element {
    let src = format!("/assets/{}.svg", square.name());
    let Block { x, y, w, h } = reversable_grid(at);

    let colors = Loc::row_colors();
    let row_colors = colors[at.into().y % colors.len()];
    let background_style = row_colors[at.into().x % row_colors.len()];

    let row = format!("grid-row: {y} / span {h};");
    let column = format!("grid-column: {x} / span {w};");

    let mouse = use_context::<Signal<Mouse<Loc>>>();
    rsx! {
        div {
            class: "overlapper",
            style: "{row}{column}",
            div {
                class: "overlapper background",
                onclick: move |_| action(Select::Pick(at)),
                onmouseover: move |_| action(Select::Consider(at)),
                onmousemove: move |evt| on_mouse_move(at, mouse, evt),
                div { class: "{background_style}" }
                ActiveHighlight { at }
                DangerHighlights { at }
            }
            WithTranslation {
                at, square, src
            }
            TargetHighlight { at }
        }
    }
}

fn on_mouse_move<Loc>(at: Loc, mut mouse: Signal<Mouse<Loc>>, evt: Event<MouseData>) {
    let (x, y) = evt.data.element_coordinates().to_tuple();
    let element = evt
        .data
        .as_web_event()
        .target()
        .unwrap()
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .clone();
    let w = element.client_width();
    let h = element.client_height();
    mouse.set(Mouse::new(x, y, w as f64, h as f64, at));
}

pub enum Select<Loc> {
    Pick(Loc),
    Consider(Loc),
    Unconsider,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Mouse<Loc> {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub cell: Loc,
}

impl<Loc> Mouse<Loc> {
    pub fn new(x: f64, y: f64, w: f64, h: f64, cell: Loc) -> Self {
        Mouse { x, y, w, h, cell }
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
        static ROWS: ColorGrid = &[&[EVEN, EXTRA_ODD], &[EXTRA_ODD, EVEN]];
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
