use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::{prelude::*, TopMsg};

pub enum Msg {
    Login,
    Signup,
    ChangeUsername(String),
    ChangePassword(String),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<TopMsg>,
}

pub struct Model {
    player: PasswordPlayer,
    loading: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            player: PasswordPlayer {
                // TODO Remove these defaults before actually deploying this...
                password: "123456789".into(),
                player: Player {
                    id: None,
                    name: "me".into(),
                },
            },
            loading: false
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login => {
                let callback = ctx.props().callback.reform(TopMsg::Login);
                self.loading = true;
                post("login", self.player.clone(), callback);
                true
            }
            Msg::Signup => {
                let callback = ctx.props().callback.reform(TopMsg::Login);
                self.loading = true;
                post("signup", self.player.clone(), callback);
                true
            }
            Msg::ChangeUsername(name) => {
                self.player.name = name;
                false
            }
            Msg::ChangePassword(password) => {
                self.player.password = password;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let login = link.callback(|_| Msg::Login);
        let signup = link.callback(|_| Msg::Signup);
        let name = link.callback(|x: Event| Msg::ChangeUsername(x.target().unwrap().unchecked_into::<HtmlInputElement>().value()));
        let password = link.callback(|x: Event| Msg::ChangePassword(x.target().unwrap().unchecked_into::<HtmlInputElement>().value()));
        html! {
            <div>
                <label for="user"><b>{ "Username" }</b></label>
                <input type="text" onchange={name} placeholder="Enter your Username" name="user"/>

                <label for="psw"><b>{ "Password" }</b></label>
                <input type="password" onchange={password} placeholder="Enter Password" name="psw"/>

                <button disabled={self.loading} onclick={login}>{ "login" }</button>
                <button disabled={self.loading} onclick={signup}>{ "signup" }</button>
            </div>
        }
    }
}
