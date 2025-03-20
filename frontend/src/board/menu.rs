use super::*;

#[component]
pub fn DrawMenuBoard(color: Color, pieces: Vec<Piece>, action: EventHandler<Piece>) -> Element {
    let squares = pieces
        .iter()
        .map(|piece| Square::Piece(color, *piece))
        .collect();
    rsx! {
        DrawBoard::<Board> {
            board: Board::from(MenuBoard::new(squares)),
            action: move |loc: Loc| {action(pieces[loc.right])},
            active: Active::NoActive,
            targets: HashSet::new(),
        }
    }
}
