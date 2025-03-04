use dioxus::prelude::*;
use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use serde::de::DeserializeOwned;

pub fn use_sse<T: DeserializeOwned + 'static>(url: String) -> Signal<Option<T>> {
    let mut holder = use_signal(|| None);
    use_future(move || {
        let mut es = EventSource::new(&url).unwrap();
        let mut stream = es.subscribe("message").unwrap();
        async move {
            while let Some(Ok((_, message))) = stream.next().await {
                let t: T = serde_json::from_str(&message.data().as_string().unwrap()).unwrap();
                holder.set(Some(t));
            }
            // We need to keep es alive until we're done with the connection
            drop(es);
        }
    });

    holder
}
