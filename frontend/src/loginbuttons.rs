use crate::prelude::*;

#[component]
pub fn login_buttons<T: 'static + Eq + Clone>(
    player: ReadOnlySignal<PasswordPlayer>,
    errors: Signal<&'static str>,
    session: Resource<T>,
) -> Element {
    let mut loading = use_signal(|| false);

    if loading() {
        spinner()
    } else {
        rsx! {
            button {
                onclick: move |_| async move {
                    loading.set(true);
                    let json = serde_json::to_string(&player()).unwrap();
                    let result = Request::post("api/login").body(json).send().await.unwrap();
                    if result.ok() {
                        session.restart();
                    } else {
                        errors.set("Invalid login credentials");
                        loading.set(false);
                    }
                },
                "Login"
            }

            button {
                onclick: move |_| async move {
                    loading.set(true);
                    let json = serde_json::to_string(&player()).unwrap();
                    let result = Request::post("api/signup").body(json).send().await.unwrap();
                    if result.ok() {
                        session.restart();
                    } else {
                        errors.set("Unable to sign up. Your username might already be taken or your password didn't have at least 8 characters.");
                        loading.set(false);
                    }
                },
                "Signup"
            }
        }
    }
}
