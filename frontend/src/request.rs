use reqwasm::http::Request;
use serde::{Serialize, de::DeserializeOwned};
use yew::Callback;

pub fn post<T: Serialize, R: DeserializeOwned + 'static>(url: &'static str, body: T, done: Callback<R>) {
    let json = serde_json::to_string(&body).unwrap();
    wasm_bindgen_futures::spawn_local(async move {
        let response = Request::post(url)
            .body(json)
            .send()
            .await.unwrap();
        let r: R = response.json().await.unwrap();
        done.emit(r);
    });
}

pub fn get<R: DeserializeOwned + 'static>(url: &'static str, done: Callback<R>) {
    wasm_bindgen_futures::spawn_local(async move {
        let response = Request::get(url)
            .send()
            .await.unwrap();
        let r: R = response.json().await.unwrap();
        done.emit(r);
    });
}
