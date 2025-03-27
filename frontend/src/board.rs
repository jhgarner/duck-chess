mod grid;
mod hex;
mod menu;
mod modifiers;
mod reversable;
mod square;
mod transition;

use game::SomeGame;
use grid::*;
use hexboard::Coord;
use hexboard::Hexboard;
use menuboard::MenuBoard;
pub use modifiers::Mods;
use modifiers::*;
use reversable::*;
use transition::*;

pub use menu::DrawMenuBoard;

pub use crate::{prelude::*, route::Route};

pub trait Drawable: ChessBoard {
    fn draw(board: Some<Self>, action: EventHandler<Self::Loc>) -> Element;
}

impl Drawable for Board {
    fn draw(board: Some<Self>, action: EventHandler<Self::Loc>) -> Element {
        square::draw(board, action)
    }
}

impl Drawable for Hexboard {
    fn draw(board: Some<Self>, action: EventHandler<Self::Loc>) -> Element {
        hex::draw(board, action)
    }
}

#[component]
pub fn DrawBoard<Board: Drawable>(
    #[props(into)] board: Some<Board>,
    action: EventHandler<Board::Loc>,
    mods: Mods<Board::Loc>,
    colors: PlayerColor,
) -> Element {
    provide_mods(mods);
    provide_locs();
    provide_rev(colors == PlayerColor::Black);
    Board::draw(board, action)
}

#[component]
pub fn DrawSomeGame(game: Game) -> Element {
    let player = use_context::<Player>();
    let colors = game.player(&player);
    match game.some_game {
        SomeGame::Square(game) => rsx! {
            DrawBoard::<Board> {
                board: game.board,
                action: |_| (),
                mods: Mods::default(),
                colors,
            }
        },

        SomeGame::Hex(game) => rsx! {
            DrawBoard::<Hexboard> {
                board: game.board,
                action: |_| (),
                mods: Mods::default(),
                colors,
            }
        },
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum Active<Loc> {
    Active(Loc),
    #[default]
    NoActive,
}

impl<Loc> From<Option<Loc>> for Active<Loc> {
    fn from(opt: Option<Loc>) -> Self {
        opt.map_or(Active::NoActive, Active::Active)
    }
}

pub fn some_game_preview(id: String, game: &Game) -> Element {
    let colors = game.player(&use_context());
    match &game.some_game {
        SomeGame::Square(game) => game_preview(id, colors, game.board.clone()),
        SomeGame::Hex(game) => game_preview(id, colors, game.board.clone()),
    }
}

pub fn game_preview<Board: Drawable>(
    id: String,
    colors: PlayerColor,
    board: impl Into<Some<Board>>,
) -> Element {
    rsx!(div {
        style: "width: 200px; height: 200px",
        Link {
            to: Route::InGame {id},
            DrawBoard::<Board> {
                action: &|_| {},
                board: board.into(),
                mods: Mods::default(),
                colors,
            }
        }
    })
}
