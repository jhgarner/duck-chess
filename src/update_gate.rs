use std::cell::Cell;
use std::rc::Rc;

use dioxus::history::provide_history_context;
use dioxus::prelude::*;

static GATED: GlobalSignal<bool> = Signal::global(|| false);

#[component]
pub fn Gate(children: Element) -> Element {
    provide_history_context(Rc::new(GatingHistory::new()));
    log::warn!("In the gate component with");
    let mut last = use_signal(|| children.clone());
    if GATED() {
        log::warn!("Was gated");
        let old_children = last();
        log::warn!("{:?}", old_children == children);
        old_children
    } else {
        log::warn!("Not gated");
        last.set(children.clone());
        children
    }
}

pub fn close_gate() {
    GATED.signal().set(true);
}

pub fn open_gate() {
    GATED.signal().set(false);
}

struct GatingHistory {
    last_route: Cell<String>,
    real: Rc<dyn History>,
}

impl GatingHistory {
    pub fn new() -> Self {
        let history = history();
        Self {
            last_route: Cell::new(history.current_route()),
            real: history,
        }
    }
}

impl History for GatingHistory {
    fn current_route(&self) -> String {
        if GATED() {
            let route = self.last_route.take();
            self.last_route.set(route.clone());
            route
        } else {
            let route = self.real.current_route();
            self.last_route.set(route.clone());
            route
        }
    }

    fn go_back(&self) {
        self.real.go_back();
    }

    fn go_forward(&self) {
        self.real.go_forward();
    }

    fn push(&self, route: String) {
        self.real.push(route);
    }

    fn replace(&self, path: String) {
        self.real.replace(path);
    }

    fn current_prefix(&self) -> Option<String> {
        self.real.current_prefix()
    }

    fn can_go_back(&self) -> bool {
        self.real.can_go_back()
    }

    fn can_go_forward(&self) -> bool {
        self.real.can_go_forward()
    }

    fn external(&self, url: String) -> bool {
        self.real.external(url)
    }

    fn updater(&self, callback: std::sync::Arc<dyn Fn() + Send + Sync>) {
        self.real.updater(callback);
    }

    fn include_prevent_default(&self) -> bool {
        self.real.include_prevent_default()
    }
}
