use dioxus::{fullstack::Streaming, prelude::*};
use serde::de::DeserializeOwned;

pub fn use_sse<
    T: DeserializeOwned + 'static + Send,
    E,
    F: Future<Output = Result<Streaming<T, E>>>,
>(
    mut holder: Signal<Option<T>>,
    callout: impl FnOnce() -> F + Clone + 'static,
) {
    use_future(move || {
        let callout = callout.clone();
        async move {
            if let Ok(mut stream) = callout().await {
                while let Some(Ok(value)) = stream.next().await {
                    holder.set(Some(value));
                }
            }
        }
    });
}
