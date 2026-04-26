use crate::prelude::*;

#[component]
pub fn login_buttons<T: 'static + Eq + Clone>(
    player: ReadSignal<PasswordPlayer>,
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
                    let result: Result<Player, _> = crate::rpc::login(player()).await;
                    if result.is_ok() {
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
                    let result = crate::rpc::signup(player()).await;
                    if result.is_ok() {
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
