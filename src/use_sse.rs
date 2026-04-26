use dioxus::{fullstack::Streaming, prelude::*};
use serde::de::DeserializeOwned;

pub fn use_sse<
    T: DeserializeOwned + 'static + Send,
    E,
    F: Future<Output = Result<Streaming<T, E>>>,
>(
    callout: impl FnOnce() -> F + Clone + 'static,
) -> Signal<Option<T>> {
    let mut holder = use_signal(|| None);
    use_future(move || {
        let callout = callout.clone();
        async move {
            if let Ok(mut stream) = callout().await {
                // let stream = stream.into_inner();
                while let Some(Ok(value)) = stream.next().await {
                    holder.set(Some(value));
                }
                // We need to keep es alive until we're done with the connection
                // drop(es);
            }
            // let mut es = EventSource::new(&url).unwrap();
            // let mut stream = es.subscribe("message").unwrap();
            // async move {}
        }
    });

    holder
}
