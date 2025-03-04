use crate::prelude::*;

pub fn spinner() -> Element {
    rsx! {
        div {
            class: "loader-inner",
            div {
                class: "loader-line-wrap",
                div {
                    class: "loader-line",
                }
            }
            div {
                class: "loader-line-wrap",
                div {
                    class: "loader-line",
                }
            }
            div {
                class: "loader-line-wrap",
                div {
                    class: "loader-line",
                }
            }
            div {
                class: "loader-line-wrap",
                div {
                    class: "loader-line",
                }
            }
            div {
                class: "loader-line-wrap",
                div {
                    class: "loader-line",
                }
            }
        }
    }
}
