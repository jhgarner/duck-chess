use crate::prelude::*;
use reqwasm::http::Request;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Notification, PushSubscriptionOptionsInit, ServiceWorkerRegistration, window};

pub fn subscribe() -> Element {
    let mut enabled = use_resource(|| async {
        let response = Request::get("/api/session_notifications")
            .send()
            .await
            .unwrap();
        response.json::<bool>().await.unwrap()
    });
    if let Some(true) = enabled.value()() {
        rsx!("You'll receive a notification on this device when it's your turn.")
    } else {
        rsx! {
            button {
                onclick: move |_| async move {
                    subscribe_me().await;
                    enabled.restart();
                },
                "Enable notifications on this device"
            }
        }
    }
}

async fn subscribe_me() {
    // Most of this method interacts with the browser API for receiving notifications
    JsFuture::from(Notification::request_permission().unwrap())
        .await
        .unwrap();
    let registration = JsFuture::from(
        window()
            .unwrap()
            .navigator()
            .service_worker()
            .register("assets/worker.js"),
    )
    .await
    .unwrap();
    let key_encoded = Request::get("api/public_key")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let options = PushSubscriptionOptionsInit::new();
    options.set_application_server_key(&JsValue::from_str(&key_encoded));
    options.set_user_visible_only(true);
    let registration = registration
        .dyn_ref::<ServiceWorkerRegistration>()
        .unwrap()
        .push_manager()
        .unwrap();
    let result = JsFuture::from(registration.subscribe_with_options(&options).unwrap())
        .await
        .unwrap();
    Request::post("api/subscribe")
        .body(js_sys::JSON::stringify(&result).unwrap().as_string())
        .send()
        .await
        .unwrap();
}
