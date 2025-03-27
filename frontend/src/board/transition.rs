use std::collections::HashMap;

use super::*;

type Locs = HashMap<SquareId, Block>;

#[component]
pub fn WithTranslation<Loc: Gridable>(at: Loc, square: Square, src: String) -> Element {
    if let Square::Piece(_, _, id) = square {
        let at_grid = reversable_grid(at);
        let old_loc = use_loc(id, at_grid).unwrap_or(at_grid);
        let (dx, dy) = diff_to(at_grid, old_loc);
        let mut movement = with_signal(Move { dx, dy, t: 0.0 });
        spawn(async move {
            post_render().await;
            movement.set(Move {
                dx: 0.0,
                dy: 0.0,
                t: (dx.powi(2) + dy.powi(2)).sqrt() / 4000.0,
            });
        });

        rsx!(SignalReader { movement, src })
    } else {
        rsx!(Keyed {
            img {
                style: "z-index: 100;",
                src
            }
        })
    }
}

#[component]
fn SignalReader(movement: Signal<Move>, src: String) -> Element {
    let Move { dx, dy, t } = movement();
    if src.contains("wQ") {
        log::warn!("Got {dx:?} {dy:?} {t:?}");
    }
    let transform = format!("transform: translate({dx}%, {dy}%);");
    let transition = format!("transition: transform {t}s ease 0s;");
    let key = src.clone();
    rsx!(Keyed {
        img {
            key,
            style: "{transform}{transition} z-index: 100;",
            src
        }
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

fn use_loc(id: SquareId, new: Block) -> Option<Block> {
    use_context::<Signal<Locs>>().write().insert(id, new)
}

fn diff_to(lhs: Block, rhs: Block) -> (f32, f32) {
    let dx = (rhs.x as f32 - lhs.x as f32) * 100.0 / rhs.w as f32;
    let dy = (rhs.y as f32 - lhs.y as f32) * 100.0 / rhs.h as f32;
    (dx, dy)
}
