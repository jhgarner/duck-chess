use std::collections::HashMap;

use hexboard::Coord;

use crate::prelude::*;

type Locs = HashMap<SquareId, Grid>;

pub struct Grid {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl From<Loc> for Grid {
    fn from(value: Loc) -> Self {
        Grid {
            x: value.right,
            y: value.down,
            w: 1,
            h: 1,
        }
    }
}

impl From<Coord> for Grid {
    fn from(coord: Coord) -> Self {
        let radius = 5;
        let col = coord.q;
        let row = 2 * coord.r + coord.q;
        let x = (col + radius) * 3 + 1;
        let y = (row + radius * 2) + 1;
        Grid {
            x: x as usize,
            y: y as usize,
            w: 4,
            h: 2,
        }
    }
}

pub trait Gridable: Into<Grid> + 'static + Copy + PartialEq {}
impl<T: Into<Grid> + 'static + Copy + PartialEq> Gridable for T {}

#[component]
pub fn WithTranslation<Loc: Gridable>(at: Loc, square: Square, children: Element) -> Element {
    if let Square::Piece(_, _, id) = square {
        let old_loc = use_loc(id, at).unwrap_or(at.into());
        let (dx, dy) = diff_to(at.into(), old_loc);
        let mut movement = with_signal(Move { dx, dy, t: 0.0 });
        spawn(async move {
            post_render().await;
            movement.set(Move {
                dx: 0.0,
                dy: 0.0,
                t: (dx.powi(2) + dy.powi(2)).sqrt() / 4000.0,
            });
        });

        rsx!(SignalReader { movement, children })
    } else {
        rsx!(div { style: "z-index: 100;", {children} })
    }
}

#[component]
fn SignalReader(movement: Signal<Move>, children: Element) -> Element {
    let Move { dx, dy, t } = movement();
    let transform = format!("transform: translate({dx}%, {dy}%);");
    let transition = format!("transition: transform {t}s ease 0s;");
    rsx!(div {
        style: "{transform}{transition} z-index: 100;",
        {children}
    })
}

async fn post_render() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Move {
    dx: f32,
    dy: f32,
    t: f32,
}

pub fn provide_locs() {
    let signal = use_signal(Locs::default);
    provide_context(signal);
}

fn use_loc<Loc: Gridable>(id: SquareId, new: Loc) -> Option<Grid> {
    use_context::<Signal<Locs>>().write().insert(id, new.into())
}

fn diff_to(lhs: Grid, rhs: Grid) -> (f32, f32) {
    let dx = (rhs.x as f32 - lhs.x as f32) * 100.0 / rhs.w as f32;
    let dy = (rhs.y as f32 - lhs.y as f32) * 100.0 / rhs.h as f32;
    (dx, dy)
}
