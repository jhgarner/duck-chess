use crate::prelude::*;

#[inline_props]
pub fn unauth<'a>(cx: Scope<'a>, session: &'a UseFuture<Option<Player>>) -> Element {
    let player = use_ref(&cx, || PasswordPlayer {
        password: "123456789".into(),
        player: Player {
            id: None,
            name: "me".into(),
        },
    });

    let loading = use_state(&cx, || false);

    let login_buttons = if *loading.current() {
        spinner()
    } else {
        rsx! {
            button {
                onclick: move |_| {
                    loading.set(true);
                    let session = (*session).clone();
                    let json = serde_json::to_string(&*player.read()).unwrap();
                    cx.push_future(async move {
                        Request::post("api/login").body(json).send().await.unwrap();
                        session.clone().restart();
                    });
                },
                "Login"
            }

            button {
                onclick: move |_| {
                    loading.set(true);
                    let session = (*session).clone();
                    let json = serde_json::to_string(&*player.read()).unwrap();
                    cx.push_future(async move {
                        Request::post("api/signup").body(json).send().await.unwrap();
                        session.restart();
                    });
                },
                "Signup"
            }
        }
    };

    cx.render(rsx! {
        div {
            class: "login",
            h1 {
                "Duck Chess"
            }
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
                    oninput: move |evt| player.write_silent().name = evt.value.clone(),
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
                    oninput: move |evt| player.write_silent().password = evt.value.clone(),
                }
            }
            div {
                login_buttons
            }
        }
    })
}
