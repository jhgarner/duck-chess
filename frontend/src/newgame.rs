use serde::de::IgnoredAny;
use bson::oid::ObjectId;

use crate::{prelude::*, TopMsg, request::get};

pub enum Msg {
    CreateGame,
    JoinGame(ObjectId),
    GotOpenGames(Vec<GameRequest>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<TopMsg>,
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
            loading: true
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GotOpenGames(games) => {
                self.open_games = games;
                self.loading = false;
            }
            Msg::CreateGame => {
                post::<_, IgnoredAny>("new_game", (), Callback::noop());
            }
            Msg::JoinGame(id) => {
                post::<_, IgnoredAny>("join_game", id, Callback::noop());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game_list = self.open_games.iter().map(|game| {
            let game = game.clone();
            let onclick = ctx.link().callback(move |_| Msg::JoinGame(game.id.unwrap()));
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

