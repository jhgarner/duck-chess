use crate::prelude::*;

pub fn with_signal<T>(val: T) -> Signal<T> {
    let mut outer = use_signal(|| None::<Signal<T>>);
    if let Some(mut signal) = *outer.peek() {
        signal.set(val);
        signal
    } else {
        let signal = Signal::new(val);
        outer.set(Some(signal));
        signal
    }
}
