use crate::{loginbuttons::login_buttons, prelude::*};

#[component]
pub fn unauth<T: 'static + Eq + Clone>(session: Resource<T>) -> Element {
    let mut player = use_signal(|| PasswordPlayer {
        password: "123456789".into(),
        player: Player {
            id: None,
            name: "me".into(),
        },
    });

    let errors = use_signal(|| "");

    rsx! {
        div {
            class: "login",
            h1 {
                "Duck Chess"
            }
            {errors}
            div {
                label {
                    "for": "user",
                    b {
                        "Username"
                    }
                }
                input {
                    "type": "text",
                    placeholder: "Enter your username",
                    name: "user",
                    oninput: move |evt| player.write().name = evt.value().clone(),
                }
            }
            div {
                label {
                    "for": "password",
                    b {
                        "Password"
                    }
                }
                input {
                    "type": "password",
                    placeholder: "Enter your password",
                    name: "password",
                    oninput: move |evt| player.write().password = evt.value().clone(),
                }
            }
            div {
                login_buttons { session, errors, player }
            }
        }
    }
}
