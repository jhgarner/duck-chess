use super::*;

pub fn draw(board: Some<Board>, action: EventHandler<Select<Loc>>) -> Element {
    let BoardId { id, .. } = use_context();
    let mut board_html: Vec<Element> = Vec::new();
    for (at, square) in board.read().iter() {
        board_html.push(rsx!(DrawBlock { at, square, action }));
    }

    let width = board.read().width();
    let height = board.read().height();
    let columns = format!("grid-template-columns: repeat({width}, 1fr);");
    let aspect_ratio = format!("aspect-ratio: {width}/{height};");
    let transition_name = format!("view-transition-name: _{id};");

    rsx!(
        Padded {
            padding: Padding::all(8),
            div {
                id: id,
                class: "board",
                style: "{columns}{aspect_ratio}{transition_name}",
                onmouseleave: move |_| action(Select::Unconsider),
                {board_html.into_iter()}
            }
        }
    )
}
