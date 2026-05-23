use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::*;

pub fn set_style(key: &'static str, css: String) {
    let Styles(styles) = consume_context();
    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(&css));
    document.head().unwrap().append_child(&style).unwrap();
    if let Some(old) = styles.borrow_mut().insert(key, style) {
        old.remove();
    }
}

pub fn clear_style(key: &'static str) {
    let Styles(styles) = consume_context();
    if let Some(old) = styles.borrow().get(key) {
        old.remove();
    }
}

pub fn use_style(key: &'static str, css: String) {
    use_effect(move || set_style(key, css.clone()));
}

#[component]
pub fn GlobalStyle(children: Element) -> Element {
    use_context_provider(Styles::default);
    children
}

#[derive(Clone, Debug, Default)]
struct Styles(Rc<RefCell<HashMap<&'static str, web_sys::Element>>>);
