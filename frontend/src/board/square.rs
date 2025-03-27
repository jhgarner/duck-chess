use super::*;

pub fn draw(board: Some<Board>, action: EventHandler<Loc>) -> Element {
    let mut board_html: Vec<Element> = Vec::new();
    for (at, square) in board.read().iter() {
        board_html.push(rsx!(DrawBlock {
            at,
            square: square,
            action: action,
        }));
    }

    let width = board.read().width();
    let height = board.read().height();
    let columns = format!("grid-template-columns: repeat({width}, 1fr);");
    let aspect_ratio = format!("aspect-ratio: {width}/{height};");

    rsx!(
        Padded {
            padding: Padding::all(8),
            div {
                class: "board",
                style: "{columns}{aspect_ratio}",
                {board_html.into_iter()}
            }
        }
    )
}
