use std::collections::HashSet;

use crate::board;
use crate::{prelude::*, request::get, TopMsg};

pub enum Msg {
    GotGames(MyGames),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<TopMsg>,
    pub player: Player,
}

pub struct Model {
    my_games: Option<MyGames>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let got_games = ctx.link().callback(Msg::GotGames);
        get("games", got_games);
        Self { my_games: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GotGames(my_games) => {
                self.my_games = Some(my_games);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(my_games) = &self.my_games {
            let my_turn_list = mk_turn_list(ctx.props(), &my_games.my_turn);
            let other_turn_list = mk_turn_list(ctx.props(), &my_games.other_turn);
            let open_games = html! {
                <div>{format!("You have {} open games.", my_games.unstarted.len())}</div>
            };
            let completed_games = mk_turn_list(ctx.props(), &my_games.completed);

            html! {
                <>
                    <button onclick={ctx.props().callback.reform(|_| TopMsg::NewGame)}>{"New Game"}</button>
                    <h1>{"Your Turn"}</h1>
                    {my_turn_list}
                    <h1>{"Their Turn"}</h1>
                    {other_turn_list}
                    <h1>{"Open Games"}</h1>
                    {open_games}
                    <h1>{"Completed Games"}</h1>
                    {completed_games}
                </>
            }
        } else {
            html! {
                <div>{"Loading your games..."}</div>
            }
        }
    }
}

fn game_preview(props: &Props, game: &Game) -> Html {
    // Not only do we need to clone game to put it in our closure, but we need to clone game each
    // time we return it because there's no "reform_once" method so Rust assumes our closure will
    // be called any number of times. I can see why we're in this pickle, but there must be a way
    // out...
    let captured_game = game.clone();
    let callback = props
        .callback
        .reform(move |_| TopMsg::InGame(captured_game.clone()));
    html! {
        <div style="width: 200px; height: 200px">
            <board::Model {callback} board={game.board.clone()} active={None} targets={HashSet::new()}/>
        </div>
    }
}

fn mk_turn_list(props: & Props, games: &[Game]) -> Html {
    games.iter().map(|game| game_preview(props, game)).collect()
}
