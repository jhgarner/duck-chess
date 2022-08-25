use bson::oid::ObjectId;
use serde::de::IgnoredAny;

use crate::{prelude::*, request::get, TopMsg};

pub enum Msg {
    CreateGame,
    JoinGame(ObjectId),
    GotOpenGames(Vec<GameRequest>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<TopMsg>,
    pub player: Player,
}

pub struct Model {
    open_games: Vec<GameRequest>,
    loading: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::GotOpenGames);
        get("open_games", callback);
        Self {
            open_games: Vec::new(),
            loading: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GotOpenGames(games) => {
                self.open_games = games;
                self.loading = false;
            }
            Msg::CreateGame => {
                let player = ctx.props().player.clone();
                let callback = ctx
                    .props()
                    .callback
                    .reform(move |_| TopMsg::Login(player.clone()));
                post::<_, IgnoredAny>("new_game", (), callback);
            }
            Msg::JoinGame(id) => {
                let player = ctx.props().player.clone();
                let callback = ctx
                    .props()
                    .callback
                    .reform(move |_| TopMsg::Login(player.clone()));
                post::<_, IgnoredAny>("join_game", id, callback);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game_list = self.open_games.iter().map(|game| {
            let game = game.clone();
            let onclick = ctx
                .link()
                .callback(move |_| Msg::JoinGame(game.id.unwrap()));
            html! {
                <button {onclick}>{game.maker.name.clone()}</button>
            }
        });

        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::CreateGame)}>{ "Create Game" }</button>
                <h1>{"Open Games:"}</h1>
                {for game_list}
            </div>
        }
    }
}
