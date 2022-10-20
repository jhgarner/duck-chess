use std::{cell::Cell, rc::Rc, sync::Arc};

use dioxus::{core::TaskId, prelude::ScopeState};
use futures::Future;

#[derive(Clone)]
pub struct UseFuture<T> {
    update: Arc<dyn Fn()>,
    needs_regen: Rc<Cell<bool>>,
    value: Option<T>,
    slot: Rc<Cell<Option<T>>>,
    task: Cell<Option<TaskId>>,
}

impl<T> UseFuture<T> {
    /// Restart the future with new dependencies.
    ///
    /// Will not cancel the previous future, but will ignore any values that it
    /// generates.
    pub fn restart(&self) {
        self.needs_regen.set(true);
        (self.update)();
    }

    /// Return any value, even old values if the future has not yet resolved.
    ///
    /// If the future has never completed, the returned value will be `None`.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

pub fn use_future<T, F>(cx: &ScopeState, future: impl Fn() -> F) -> &UseFuture<T>
where
    T: 'static,
    F: 'static + Future<Output = T>,
{
    let state = cx.use_hook(move |_| UseFuture {
        update: cx.schedule_update(),
        needs_regen: Rc::new(Cell::new(true)),
        slot: Rc::new(Cell::new(None)),
        value: None,
        task: Cell::new(None),
    });

    if let Some(value) = state.slot.take() {
        state.value = Some(value);
        state.task.set(None);
    }

    if state.needs_regen.get() {
        // We don't need regen anymore
        state.needs_regen.set(false);

        // Create the new future
        let fut = future();

        // Clone in our cells
        let slot = state.slot.clone();
        let schedule_update = state.update.clone();

        // Cancel the current future
        if let Some(current) = state.task.take() {
            cx.remove_future(current);
        }

        state.task.set(Some(cx.push_future(async move {
            let res = fut.await;
            slot.set(Some(res));
            schedule_update();
        })));
    }

    state
}
