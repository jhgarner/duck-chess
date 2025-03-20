use std::fmt::Debug;

use crate::prelude::*;

#[derive(Debug)]
struct AlwaysSignal<T: 'static> {
    signal: Option<Signal<T>>,
}

impl<T: 'static> Clone for AlwaysSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for AlwaysSignal<T> {}

pub fn use_always<T: Debug>(val: T) -> Signal<T> {
    let mut always = use_hook(|| AlwaysSignal { signal: None });
    if let Some(mut signal) = always.signal {
        signal.set(val);
        signal
    } else {
        let signal = Signal::new(val);
        always.signal = Some(signal);
        signal
    }
}
