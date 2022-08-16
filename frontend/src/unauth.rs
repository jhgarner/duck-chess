use crate::{prelude::*, TopMsg};

pub enum Msg {
    Login,
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div>
                <label for="user"><b>{ "Username" }</b></label>
                <input type="text" placeholder="Enter your Username" name="user"/>

                <label for="psw"><b>{ "Password" }</b></label>
                <input type="password" placeholder="Enter Password" name="psw"/>

                <button disabled={self.loading} onclick={link.callback(|_| Msg::Login)}>{ "login" }</button>
            </div>
        }
    }
}
