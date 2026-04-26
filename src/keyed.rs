use std::iter::once;

use crate::prelude::*;

#[component]
pub fn Keyed(children: Element) -> Element {
    rsx!({ once(children) })
}
