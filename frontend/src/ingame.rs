use std::collections::HashSet;

use gloo_events::EventListener;
use serde::de::IgnoredAny;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, EventSourceInit, MessageEvent};

use crate::board;
use crate::{prelude::*, TopMsg};

pub enum Msg {
    Clicked(Loc),
    Update(Game),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<TopMsg>,
    pub game: Game,
    pub player: Player,
}

#[derive(PartialEq, Clone)]
pub enum GameState {
    GameEnd(Color),
    OtherMove,
    MyMove,
    Selected(Loc, Vec<Action>),
    Promotion(Loc, Rel, Board),
    PlacingDuck(Loc, Action),
}

pub struct Model {
    game_state: GameState,
    game: Game,
    el: EventListener,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        // TODO Make the event source better. It randomly doesn't connect correctly and belong in
        // its own file/abstraction.
        let game = ctx.props().game.clone();
        let es = EventSource::new_with_event_source_init_dict(
            &format!("poll/{}", game.id.unwrap().to_hex()),
            EventSourceInit::new().with_credentials(true),
        )
        .unwrap();
        let callback = ctx.link().callback(|m| m);
        let el = EventListener::new(&es, "message", move |event| {
            let text = event
                .dyn_ref::<MessageEvent>()
                .unwrap()
                .data()
                .as_string()
                .unwrap();
            let game: Game = serde_json::from_str(&text).unwrap();
            callback.emit(Msg::Update(game));
        });

        Self {
            game_state: get_game_state(&game, &ctx.props().player),
            game: ctx.props().game.clone(),
            el,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let game = &self.game;
        match msg {
            Msg::Update(game) => {
                self.game_state = get_game_state(&game, &ctx.props().player);
                self.game = game;
            }
            Msg::Clicked(loc) => match &self.game_state {
                GameState::OtherMove | GameState::GameEnd(_) => {}
                GameState::MyMove => {
                    let valid_moves = game.valid_locations(loc);
                    self.game_state = GameState::Selected(loc, valid_moves);
                }
                GameState::Selected(start, valid_moves) => {
                    if let Some(action) = valid_moves
                        .iter()
                        .find(|action| action.get_target(game).from(*start) == loc)
                    {
                        if let Action::Promote(rel, _) = action {
                            let board = game.mk_promotion_board();
                            self.game_state = GameState::Promotion(*start, *rel, board);
                        } else {
                            self.game_state = GameState::PlacingDuck(*start, *action);
                        }
                    } else {
                        self.game_state = GameState::MyMove;
                    }
                }
                GameState::Promotion(start, rel, board) => {
                    let action = Action::Promote(*rel, board.grid[0][loc.right]);
                    self.game_state = GameState::PlacingDuck(*start, action)
                }
                GameState::PlacingDuck(start, action) => {
                    let mut new_game = game.clone();
                    new_game.apply(*start, *action);
                    if new_game.valid_duck(loc) {
                        new_game.apply_duck(loc);
                        let turn = WithId {
                            id: game.id.unwrap(),
                            t: Turn {
                                from: *start,
                                action: *action,
                                duck_to: loc,
                            },
                        };
                        post::<_, IgnoredAny>("turn", turn, Callback::noop());
                        ctx.link().send_message(Msg::Update(new_game));
                        self.game_state = GameState::OtherMove;
                    } else {
                        let valid_moves = game.valid_locations(loc);
                        self.game_state = GameState::Selected(loc, valid_moves);
                    }
                }
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game = &self.game;
        let (board, active, targets): (_, _, HashSet<_>) = match &self.game_state {
            GameState::Selected(start, actions) => {
                let targets = actions
                    .iter()
                    .map(|action| action.get_target(game).from(*start))
                    .collect();
                (game.board.clone(), Some(*start), targets)
            }
            GameState::Promotion(_, _, board) => (board.clone(), None, HashSet::new()),
            GameState::PlacingDuck(start, action) => {
                let mut new_game = game.clone();
                new_game.apply(*start, *action);
                let targets = new_game.board.empties().collect();
                let duck = new_game.board.duck();
                (new_game.board, duck, targets)
            }
            GameState::MyMove | GameState::OtherMove | GameState::GameEnd(_) => {
                (game.board.clone(), None, HashSet::new())
            }
        };

        let callback = ctx.link().callback(Msg::Clicked);
        let header = if let GameState::GameEnd(color) = self.game_state {
            html! {
                <h1>{format!("{:?} Won!", color)}</h1>
            }
        }else {
            html! {}
        };

        html! {
            <>
                {header}
                // TODO So I want the board to take up all of the remaining space without changing
                // proportion. So if there's 100px of available height and 200 px of available
                // width, the board should be 100px tall and 100px wide. If the board were a
                // non-square size, it should apply the same rule: scale up to fill the space
                // without making scrollbars appear.
                <div style="width: 500px; height: 500px">
                    <board::Model {callback} {board} {active} targets={targets}/>
                </div>
            </>
        }
    }
}

fn get_game_state(game: &Game, player: &Player) -> GameState {
    if game.player(player).contains(&game.turn()) {
        GameState::MyMove
    } else if let Some(color) = game.game_over() {
        GameState::GameEnd(color)
    } else {
        GameState::OtherMove
    }
}
