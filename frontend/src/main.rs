mod board;
mod ingame;
mod mainmenu;
mod newgame;
mod prelude;
mod preview;
mod request;
mod unauth;

use prelude::*;

pub enum TopMsg {
    Login(Player),
    InGame(Game),
    NewGame,
    Logout,
}

enum Model {
    Loading,
    Unauth,
    Auth(Player, AppState),
}
enum AppState {
    MainMenu,
    InGame(Game),
    NewGame,
}

impl Component for Model {
    type Message = TopMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        post(
            "session",
            (),
            ctx.link().callback(|player: Option<Player>| {
                if let Some(player) = player {
                    TopMsg::Login(player)
                } else {
                    TopMsg::Logout
                }
            }),
        );

        Model::Loading
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TopMsg::Login(player) => *self = Model::Auth(player, AppState::MainMenu),
            TopMsg::InGame(game) => {
                if let Model::Auth(_, state) = self {
                    *state = AppState::InGame(game);
                }
            }
            TopMsg::NewGame => {
                if let Model::Auth(_, state) = self {
                    *state = AppState::NewGame;
                }
            }
            TopMsg::Logout => *self = Model::Unauth,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(|msg| msg);
        match self {
            Model::Loading => html! {
                <h1>{"Loading..."}</h1>
            },
            Model::Unauth => html! {
                <unauth::Model {callback} />
            },
            Model::Auth(player, state) => match state {
                AppState::MainMenu => html! {
                    // I don't love the amount of cloning that Yew requires. The player used by the
                    // inner components should be a reference to the one held in the model. Adding
                    // lifetimes to the Props struct means adding lifetimes to the Model struct
                    // which means phantom data and adding explicit lifetimes all over the place...
                    // I'd be willing to see if it works though.
                    <mainmenu::Model {callback} player={player.clone()} />
                },
                AppState::InGame(game) => html! {
                    <ingame::Model {callback} player={player.clone()} game={game.clone()}/>
                },
                AppState::NewGame => html! {
                    <newgame::Model {callback} player={player.clone()} />
                },
            },
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
