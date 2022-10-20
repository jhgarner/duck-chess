use crate::board;
use crate::{board::Active, prelude::*};

#[derive(PartialEq, Clone)]
pub enum GameState {
    Waiting,
    MyMove,
    Selected(Loc, Vec<Action>),
    // TODO I'd rather store the promotion boards in 'static
    Promotion(Loc, Rel, Board),
    PlacingDuck(Loc, Action),
}

#[inline_props]
pub fn active_game<'a>(
    cx: Scope<'a>,
    id: ObjectId,
    og_game: Game,
    updated: &'a Cell<bool>,
) -> Element {
    let clicked = cx.use_hook(|_| Cell::new(None));
    let game = cx.use_hook(|_| og_game.clone());
    let game_state = cx.use_hook(|_| GameState::MyMove);

    // I almost opened a bug report on this, but then checked and React acts the same way. If the
    // og_game changes, it doesn't override the local hook. This Cell trick tells the app when to
    // accept the new og_game state.
    if updated.replace(false) {
        *game = og_game.clone();
        *game_state = GameState::MyMove;
    }

    if let Some(loc) = clicked.take() {
        update(&cx, *id, game, game_state, loc)
    }

    let (board, active, targets): (_, _, HashSet<_>) = match game_state {
        GameState::Selected(start, actions) => {
            let targets = actions
                .iter()
                .map(|action| action.get_target(game).from(*start))
                .collect();
            (&game.board, Active::Active(*start), targets)
        }
        GameState::Promotion(_, _, board) => (board, Active::NoActive, HashSet::new()),
        GameState::PlacingDuck(_, _) => {
            let targets = game.board.empties().collect();
            let duck = game.board.duck().into();
            (&game.board, duck, targets)
        }
        GameState::MyMove | GameState::Waiting => (&game.board, Active::NoActive, HashSet::new()),
    };

    cx.render(rsx! {
        board::board {
            action: move |loc| {
                clicked.set(Some(loc));
                cx.schedule_update()();
            },
            board: Cow::Borrowed(board),
            active: active,
            targets: targets,
        }
    })
}

fn update(cx: &ScopeState, id: ObjectId, game: &mut Game, game_state: &mut GameState, loc: Loc) {
    let new_game_state = match game_state {
        GameState::Waiting => GameState::Waiting,
        GameState::MyMove => {
            let valid_moves = game.valid_locations(loc);
            GameState::Selected(loc, valid_moves)
        }
        GameState::Selected(start, valid_moves) => {
            if let Some(action) = valid_moves
                .iter()
                .find(|action| action.get_target(game).from(*start) == loc)
            {
                if let Action::Promote(rel, _) = action {
                    let board = game.mk_promotion_board();
                    GameState::Promotion(*start, *rel, board)
                } else {
                    Game::apply(game, *start, *action);
                    GameState::PlacingDuck(*start, *action)
                }
            } else {
                GameState::MyMove
            }
        }
        GameState::Promotion(start, rel, board) => {
            let action = Action::Promote(*rel, board.grid[0][loc.right]);
            Game::apply(game, loc, action);
            GameState::PlacingDuck(*start, action)
        }
        GameState::PlacingDuck(start, action) => {
            if game.valid_duck(loc) {
                game.apply_duck(loc);
                let turn = WithId {
                    id,
                    t: Turn {
                        from: *start,
                        action: *action,
                        duck_to: loc,
                    },
                };
                game.turns.push(turn.t);
                cx.push_future(async move {
                    let json = serde_json::to_string(&turn).unwrap();
                    Request::post("/api/turn").body(json).send().await.unwrap();
                });
                GameState::Waiting
            } else {
                GameState::PlacingDuck(*start, *action)
            }
        }
    };
    *game_state = new_game_state;
}
