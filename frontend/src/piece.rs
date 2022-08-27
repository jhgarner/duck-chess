use wasm_bindgen::JsValue;
use web_sys::{window, KeyframeEffect, Animation, console};
use yew::{function_component, Properties};
use crate::prelude::*;


#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: Option<String>,
    pub square: Square,
}

#[function_component(Model)]
pub fn piece(props: &Props) -> Html {
    let piece = format!("assets/{}.svg", props.square.name());
    if let Some(id) = &props.id {
        console::log_1(&JsValue::from_str(&format!("RENDERING")));
        let body = window().unwrap().document().unwrap().body();
        let id = format!("_{}_{}", id, props.square.name());
        let selector = format!("#{}", id);
        if let Some(element) = body.as_ref().unwrap().query_selector(&selector).unwrap() {
            let old_rect = element.get_bounding_client_rect();
            console::log_1(&JsValue::from_str(&format!("id: {id}, old: {:?}", old_rect.left())));
            use_effect(move || {
                let new_rect = body.unwrap().query_selector(&selector).unwrap().unwrap().get_bounding_client_rect();
                console::log_1(&JsValue::from_str(&format!("old: {:?}, new: {:?}", old_rect.left(), new_rect.left())));
                let delta_left = new_rect.left() - old_rect.left();
                let delta_top = new_rect.top() - old_rect.top();
                let transform = format!(r#"{{"transform": "translate({}px, {}px)"}}"#, delta_left, delta_top);
                let done = r#"{"transform": "translate(5px, 5px)"}"#;
                let frames = js_sys::JSON::parse(&format!(r#"[{transform}, {done}]"#)).unwrap();
                let effect = KeyframeEffect::new_with_opt_element_and_keyframes_and_f64(Some(&element), Some(&frames.into()), 10000.0).unwrap();
                let animation = Animation::new_with_effect_and_timeline(Some(&effect), Some(&window().unwrap().document().unwrap().timeline())).unwrap();
                animation.play().unwrap();
                || {}
            });
        };
        html! {
            <img src={piece} {id}/>
        }
    } else {
        html! {
            <img src={piece}/>
        }
    }
}
