use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::*;

#[component]
pub fn DrawBlock<Loc: Gridable>(
    square: Square,
    at: Loc,
    action: EventHandler<Select<Loc>>,
) -> Element {
    let src = format!("/assets/{}.svg", square.name());
    let block @ Block { x, y, w, h, style } = reversable_grid(at);

    let row = format!("grid-row: {y} / span {h};");
    let column = format!("grid-column: {x} / span {w};");

    let mouse = use_context::<Signal<Mouse>>();
    rsx! {
        div {
            class: "overlapper",
            style: "{row}{column}",
            div {
                class: "overlapper background",
                onclick: move |_| action(Select::Pick(at)),
                onmouseover: move |_| action(Select::Consider(at)),
                onmousemove: move |evt| on_mouse_move(block, mouse, evt),
                div { class: "{style}" }
                ActiveHighlight { block }
                DangerHighlights { block }
            }
            WithTranslation {
                block, square, src
            }
            TargetHighlight { block }
        }
    }
}

fn on_mouse_move(at: Block, mut mouse: Signal<Mouse>, evt: Event<MouseData>) {
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
pub struct Mouse {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub cell: Block,
}

impl Mouse {
    pub fn new(x: f64, y: f64, w: f64, h: f64, cell: Block) -> Self {
        Mouse { x, y, w, h, cell }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Block {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub style: &'static str,
}

impl From<Loc> for Block {
    fn from(value: Loc) -> Self {
        let x = value.right + 1;
        let y = value.down + 1;
        static COLORS: ColorGrid = &[&[EVEN, EXTRA_ODD], &[EXTRA_ODD, EVEN]];
        let row_colors = COLORS[y % COLORS.len()];
        let style = row_colors[x % row_colors.len()];
        Block {
            x,
            y,
            w: 1,
            h: 1,
            style,
        }
    }
}

impl From<Coord> for Block {
    fn from(coord: Coord) -> Self {
        static COLORS: ColorGrid = &[&[EVEN], &[ODD], &[EXTRA_ODD]];
        let radius = 5;
        let col = coord.q;
        let row = 2 * coord.r + coord.q;
        let x = ((col + radius) * 3 + 1) as usize;
        let y = ((row + radius * 2) + 1) as usize;
        let row_colors = COLORS[y % COLORS.len()];
        let style = row_colors[x % row_colors.len()];
        Block {
            x,
            y,
            w: 4,
            h: 2,
            style,
        }
    }
}

pub trait Gridable: Into<Block> + 'static + Copy + PartialEq + Eq + Hash {
    fn height() -> usize;
}

static EVEN: &str = "cellEven";
static ODD: &str = "cellOdd";
static EXTRA_ODD: &str = "cellExtraOdd";

impl Gridable for Loc {
    fn height() -> usize {
        9
    }
}

impl Gridable for Coord {
    fn height() -> usize {
        22
    }
}

type ColorGrid = &'static [&'static [&'static str]];
