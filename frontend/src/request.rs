use futures::stream::StreamExt;
use reqwest_eventsource::{Event, RequestBuilderExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{sync::oneshot::{Sender, self}, select};

use crate::app::{Message, MessageChannel};

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    message: MessageChannel,
}

const ROOT: &'static str = "http://127.0.0.1:8000";

fn mk_url(path: &'static str) -> String {
    format!("{ROOT}/{path}")
}

pub struct EventStream {
    cancel: Option<Sender<()>>,
}

impl EventStream {
    pub fn new(sender: Sender<()>) -> EventStream {
        EventStream {
            cancel: Some(sender)
        }
    }

    pub fn cancel(&mut self) {
        if let Some(sender) = std::mem::take(&mut self.cancel) {
            // TODO handle losing connection more gracefully
            sender.send(()).unwrap();
        }
    }
}

impl Client {
    pub fn new(message: MessageChannel) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        Client { client, message }
    }
}

impl Client {
    pub fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &'static str,
        body: &T,
        done: impl FnOnce(R) -> Message + Send + 'static,
    ) {
        let body = serde_json::to_string(&body).unwrap();
        let this = self.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let result = this
                    .client
                    .post(mk_url(path))
                    .body(body)
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                let r = serde_json::from_str::<'_, R>(&result).unwrap();
                this.message.write(done(r));
            })
        });
    }

    pub fn get<R: DeserializeOwned>(
        &self,
        path: &'static str,
        done: impl FnOnce(R) -> Message + Send + 'static,
    ) {
        let this = self.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let result = this
                    .client
                    .get(mk_url(path))
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                let r = serde_json::from_str::<'_, R>(&result).unwrap();
                this.message.write(done(r));
            })
        });
    }

    pub fn poll<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &'static str,
        body: &T,
        on_event: impl Fn(R) -> Message + Send + 'static,
    ) -> EventStream {
        let body = serde_json::to_string(&body).unwrap();
        let this = self.clone();
        let mut es = this
            .client
            .post(mk_url(path))
            .body(body)
            .eventsource()
            .unwrap();
        let (tx, mut rx) = oneshot::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                loop {
                    select! {
                        event = es.next() => {
                            match event {
                                Some(Ok(Event::Message(message))) => {
                                    let r: R = serde_json::from_str(&message.data).unwrap();
                                    this.message.write(on_event(r));
                                }
                                None | Some(Err(_)) => break,
                                _ => {}
                            }
                        }
                        _ = &mut rx => break
                    }
                }
                es.close();
            });
        });

        EventStream::new(tx)
    }
}
