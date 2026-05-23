use super::*;

#[component]
pub fn Specular(x: f64, y: f64, effect: Effect, color: &'static str) -> Element {
    let class = effect.class();
    rsx!(div {
        class: "specular {class}",
        style: "--color: var(--{color}); --x: {x}px; --y: {y}px;",
    })
}
