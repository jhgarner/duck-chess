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
            action: move |select| act(&pieces, select, action),
            mods: Mods::default(),
            colors: PlayerColor::None
        }
    }
}

fn act(pieces: &Vec<Piece>, select: Select<Loc>, action: EventHandler<Piece>) {
    if let Select::Pick(loc) = select {
        action(pieces[loc.right]);
    }
}
