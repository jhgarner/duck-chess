use crate::prelude::*;

#[component]
pub fn Padded(padding: Padding, children: Element) -> Element {
    let Padding {
        top,
        bottom,
        left,
        right,
    } = padding;
    rsx! {
        div {
            class: "padding",
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
                style: "height: 100%; width: 100%; overflow: hidden",
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
    pub fn vert(px: u32) -> Padding {
        Padding {
            top: px,
            bottom: px,
            left: 0,
            right: 0,
        }
    }

    pub fn all(px: u32) -> Padding {
        Padding {
            top: px,
            bottom: px,
            left: px,
            right: px,
        }
    }
}
