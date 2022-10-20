use std::{cell::Cell, rc::Rc};

use dioxus::prelude::ScopeState;
use gloo_events::EventListener;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, EventSourceInit, MessageEvent};

pub fn use_event_listener<T: DeserializeOwned + 'static>(
    cx: &ScopeState,
    url: String,
) -> &Cell<Option<T>> {
    let holder = cx.use_hook(|_| Rc::new(Cell::new(None)));
    let _ = cx.use_hook(|_| {
        let mut es_init = EventSourceInit::new();
        es_init.with_credentials(true);
        let es = EventSource::new_with_event_source_init_dict(&url, &es_init).unwrap();
        let schedule = cx.schedule_update();
        let holder = holder.clone();
        EventListener::new(&es, "message", move |event| {
            let text = event
                .dyn_ref::<MessageEvent>()
                .unwrap()
                .data()
                .as_string()
                .unwrap();
            let t: T = serde_json::from_str(&text).unwrap();
            holder.set(Some(t));
            schedule();
        })
    });

    holder
}
