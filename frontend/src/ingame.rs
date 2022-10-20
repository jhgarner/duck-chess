use crate::board::{self, Active};
use crate::prelude::*;
use crate::{activegame, joinablegame};

#[inline_props]
pub fn in_game<'a>(cx: Scope<'a>, player: &'a Player) -> Element {
    let id = use_route(&cx).segment("id").unwrap();
    let game_or_request = use_event_listener::<Option<AnyGame>>(&cx, format!("/api/poll/{id}"));

    // This forces the active game view to accept the updated state
    let updated = cx.use_hook(|_| Cell::new(true));
    updated.set(true);

    cx.render(match game_or_request.take() {
        None => rsx! {div {}},
        Some(None) => rsx! { "Invalid game id" },
        Some(Some(with_id)) => match with_id.game {
            GameOrRequest::Request(request) => {
                rsx! {
                    [if request.maker.id == player.id {
                        "Your game hasn't started yet. Share this page to invite someone."
                    } else {
                        ""
                    }]
                    joinablegame::joinable_game {
                        request: request,
                        id: with_id.id.unwrap(),
                    }
                }
            }
            GameOrRequest::Game(game) | GameOrRequest::Completed(game) => {
                match get_game_state(&game, player) {
                    TurnState::MyTurn => {
                        rsx! {
                            activegame::active_game {
                                id: with_id.id.unwrap(),
                                og_game: game,
                                updated: updated,
                            }
                        }
                    }
                    TurnState::OtherTurn => rsx! {
                        "It is not your turn",
                        board::board {
                            action: &|_| {},
                            active: Active::NoActive,
                            board: Cow::Owned(game.board),
                            targets: HashSet::new()
                        }
                    },
                    TurnState::Ended(winner) => rsx! {
                        [format!("{:?} Won!", winner)],
                        board::board {
                            action: &|_| {},
                            active: Active::NoActive,
                            board: Cow::Owned(game.board),
                            targets: HashSet::new()
                        }
                    },
                }
            }
        },
    })
}

enum TurnState {
    MyTurn,
    OtherTurn,
    Ended(Color),
}

fn get_game_state(game: &Game, player: &Player) -> TurnState {
    if let Some(color) = game.game_over() {
        TurnState::Ended(color)
    } else if game.player(player).contains(&game.turn()) {
        TurnState::MyTurn
    } else {
        TurnState::OtherTurn
    }
}
