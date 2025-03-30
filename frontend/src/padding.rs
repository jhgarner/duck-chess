use crate::prelude::*;

#[component]
pub fn Padded(
    padding: Padding,
    #[props(default = "".to_string())] class: String,
    #[props(default = "".to_string())] style: String,
    children: Element,
) -> Element {
    let Padding {
        top,
        bottom,
        left,
        right,
    } = padding;
    rsx! {
        div {
            class: "padding {class}",
            style,
            div {
                style: "height: {top}px; width: {left}px;",
            }
            div {
                style: "height: {top}px;",
            }
            div {
                style: "height: {top}px; width: {right}px;",
            }
            div {
                style: "width: {left}px;",
            }
            div {
                style: "display: grid; height: 100%; width: 100%; overflow: hidden",
                {children}
            }
            div {
                style: "width: {right}px;",
            }
            div {
                style: "height: {bottom}px; width: {left}px;",
            }
            div {
                style: "height: {bottom}px;",
            }
            div {
                style: "height: {bottom}px; width: {right}px;",
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Padding {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

impl Padding {
    pub fn all(px: u32) -> Padding {
        Padding {
            top: px,
            bottom: px,
            left: px,
            right: px,
        }
    }
}
