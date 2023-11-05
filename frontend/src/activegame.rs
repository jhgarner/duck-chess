use std::collections::HashMap;

use common::board::{ChessBoard, ChessBoardMut};
use common::game::GameRaw;

use crate::board;
use crate::{board::Active, prelude::*};

#[derive(PartialEq, Eq, Clone)]
pub enum GameState<Board: ChessBoard> {
    Waiting,
    MyMove,
    Selected(Board::Loc, HashMap<Board::Loc, ActionRaw<Board::Rel>>),
    Promotion(Board::Loc, Board::Rel),
    PlacingDuck(Board::Loc, ActionRaw<Board::Rel>),
}

#[derive(Props)]
struct ActiveGameProps<'a, Board: ChessBoard> {
    id: ObjectId,
    og_game: GameRaw<Board>,
    updated: &'a Cell<bool>,
}

pub fn active_game<'a, Board: ChessBoardMut<Board = Board> + Clone>(
    cx: Scope<'a, ActiveGameProps<'a, Board>>,
) -> Element {
    let ActiveGameProps {
        id,
        og_game,
        updated,
    } = cx.props;
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
        *game_state = update(&cx, *id, game, game_state, loc)
    }

    let (board, active, targets): (_, _, HashSet<_>) = match game_state {
        GameState::Selected(start, actions) => {
            let targets = actions.keys().collect();
            (Cow::Borrowed(&game.board), Active::Active(*start), targets)
        }
        GameState::Promotion(_, _) => (
            Cow::Owned(game.mk_promotion_board()),
            Active::NoActive,
            HashSet::new(),
        ),
        GameState::PlacingDuck(_, _) => {
            let targets = game.board.empties().collect();
            let duck = game.duck_loc.into();
            (Cow::Borrowed(&game.board), duck, targets)
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

fn update<Board: ChessBoardMut>(
    cx: &ScopeState,
    id: ObjectId,
    game: &mut GameRaw<Board>,
    game_state: &GameState<Board>,
    loc: Board::Loc,
) -> GameState<Board> {
    match game_state {
        GameState::Waiting => GameState::Waiting,
        GameState::MyMove => {
            let valid_moves = game.valid_locations(loc);
            if valid_moves.is_empty() {
                GameState::MyMove
            } else {
                GameState::Selected(loc, valid_moves)
            }
        }
        GameState::Selected(start, valid_moves) => {
            if let Some(action) = valid_moves.get(&loc) {
                if let ActionRaw::Promote(rel, _) = action {
                    GameState::Promotion(*start, *rel)
                } else {
                    game.apply(*start, *action);
                    GameState::PlacingDuck(*start, *action)
                }
            } else {
                update(cx, id, game, &GameState::MyMove, loc)
            }
        }
        GameState::Promotion(start, rel) => {
            let action = ActionRaw::Promote(*rel, GameRaw::mk_promotion_pieces()[loc.right]);
            game.apply(*start, action);
            GameState::PlacingDuck(*start, action)
        }
        GameState::PlacingDuck(start, action) => {
            if game.valid_duck(loc) {
                game.apply_duck(loc);
                let turn = WithId {
                    id,
                    t: TurnRaw {
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
    }
}
