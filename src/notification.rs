use crate::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Notification, PushSubscriptionOptionsInit, ServiceWorkerRegistration, window};

pub fn subscribe() -> Element {
    let mut enabled =
        use_resource(|| async { crate::rpc::notifications_enabled().await.unwrap_or(false) });
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
            .register(&asset!("assets/worker.js").to_string()),
    )
    .await
    .unwrap();
    let key_encoded = crate::rpc::public_key_rpc().await.unwrap();
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
    crate::rpc::subscribe_rpc(
        js_sys::JSON::stringify(&result)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .await
    .unwrap();
}
