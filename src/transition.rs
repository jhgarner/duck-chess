use dioxus::core::Callback;
use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{StartViewTransitionOptions, ViewTransition, window};

pub struct Resolver(Function);

impl Resolver {
    pub fn finish(self) {
        self.0.call0(&JsValue::undefined()).unwrap();
    }
}

pub struct Transition(ViewTransition);

impl Transition {
    pub fn on_complete(&self, f: impl FnOnce() + 'static) {
        let mut f = Some(f);
        let finished = self.0.finished();
        spawn_local(async move {
            let _ = finished.await;
            f.take().unwrap()();
        });
    }

    pub fn on_complete_callback(&self, f: impl FnOnce() + 'static) {
        let mut f = Some(f);
        let callback = Callback::new(move |()| {
            f.take().unwrap()();
        });
        self.on_complete(move || {
            callback.call(());
        });
    }
}

pub fn transition_callback(ready: impl FnOnce() + 'static) -> Transition {
    let mut ready = Some(ready);
    let callback = Callback::new(move |resolver: Resolver| {
        ready.take().unwrap()();
        resolver.finish();
    });
    transition(move |resolver| {
        callback.call(resolver);
    })
}

pub fn transition(ready: impl FnOnce(Resolver) + 'static) -> Transition {
    let mut ready = Some(ready);
    let options = StartViewTransitionOptions::new();
    let mut resolver = None;
    let promise = Promise::new(&mut |resolve, _reject| {
        resolver = Some(resolve);
    });

    let closure: ScopedClosure<'static, dyn FnMut() -> Promise> = ScopedClosure::own(move || {
        let resolver = resolver.clone();
        ready.take().unwrap()(Resolver(resolver.unwrap()));
        promise.clone()
    });
    options.set_update(Some(&Function::from_closure(closure)));
    let vt = window()
        .unwrap()
        .document()
        .unwrap()
        .start_view_transition_with_start_view_transition_options(&options)
        .unwrap();
    Transition(vt)
}
