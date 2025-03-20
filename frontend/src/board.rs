mod hex;
mod menu;
mod square;

use game::SomeGame;
use hexboard::Hexboard;
use menuboard::MenuBoard;

pub use menu::DrawMenuBoard;

pub use crate::{prelude::*, route::Route};

pub trait Drawable: ChessBoard {
    fn draw(
        board: Some<Self>,
        action: EventHandler<Self::Loc>,
        active: Active<Self::Loc>,
        targets: HashSet<Self::Loc>,
    ) -> Element;
}

impl Drawable for Board {
    fn draw(
        board: Some<Self>,
        action: EventHandler<Self::Loc>,
        active: Active<Self::Loc>,
        targets: HashSet<Self::Loc>,
    ) -> Element {
        square::draw(board, action, active, targets)
    }
}

impl Drawable for Hexboard {
    fn draw(
        board: Some<Self>,
        action: EventHandler<Self::Loc>,
        active: Active<Self::Loc>,
        targets: HashSet<Self::Loc>,
    ) -> Element {
        hex::draw(board, action, active, targets)
    }
}

#[component]
pub fn DrawBoard<Board: Drawable>(
    #[props(into)] board: Some<Board>,
    action: EventHandler<Board::Loc>,
    active: Active<Board::Loc>,
    targets: HashSet<Board::Loc>,
) -> Element {
    Board::draw(board, action, active, targets)
}

#[component]
pub fn DrawSomeGame(some_game: SomeGame) -> Element {
    match some_game {
        SomeGame::Square(game) => rsx! {
            DrawBoard::<Board> {
                board: game.board,
                action: |_| (),
                active: Active::NoActive,
                targets: HashSet::new()
            }
        },

        SomeGame::Hex(game) => rsx! {
            DrawBoard::<Hexboard> {
                board: game.board,
                action: |_| (),
                active: Active::NoActive,
                targets: HashSet::new()
            }
        },
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Active<Loc> {
    Active(Loc),
    NoActive,
}

impl<Loc> From<Option<Loc>> for Active<Loc> {
    fn from(opt: Option<Loc>) -> Self {
        opt.map_or(Active::NoActive, Active::Active)
    }
}

pub fn some_game_preview(id: String, some_game: &SomeGame) -> Element {
    match some_game {
        SomeGame::Square(game) => game_preview(id, game.board.clone()),
        SomeGame::Hex(game) => game_preview(id, game.board.clone()),
    }
}

pub fn game_preview<Board: Drawable>(id: String, board: impl Into<Some<Board>>) -> Element {
    rsx!(div {
        style: "width: 200px; height: 200px",
        Link {
            to: Route::InGame {id},
            DrawBoard::<Board> {
                action: &|_| {},
                board: board.into(),
                active: Active::NoActive,
                targets: HashSet::new(),
            }
        }
    })
}
