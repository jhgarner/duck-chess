use super::*;

#[component]
pub fn Bordered(x: f64, y: f64, effect: Effect, color: String, children: Element) -> Element {
    let class = effect.class();
    rsx!(
        Padded {
            class: "border {class}",
            style: "--x: {x}px; --y: {y}px; --color: var(--{color})",
            padding: Padding::all(2),
            div {
                style: "background: var(--bg); clip-path: var(--clip)",
            }
        }
        {children}
    )
}
