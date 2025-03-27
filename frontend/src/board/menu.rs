use super::*;

#[component]
pub fn DrawMenuBoard(color: Color, pieces: Vec<Piece>, action: EventHandler<Piece>) -> Element {
    let squares = pieces
        .iter()
        .map(|piece| Square::piece(color, *piece))
        .collect();
    rsx! {
        DrawBoard::<Board> {
            board: Board::from(MenuBoard::new(squares)),
            action: move |loc: Loc| {action(pieces[loc.right])},
            mods: Mods::default(),
            colors: PlayerColor::None
        }
    }
}
