use super::*;

pub fn draw(board: Some<Hexboard>, action: EventHandler<Select<Coord>>) -> Element {
    let BoardId { id, .. } = use_context();
    let mut board_html: Vec<Element> = Vec::new();
    for (at, square) in board.read().iter() {
        board_html.push(rsx!(DrawBlock { at, square, action }));
    }

    let diameter = 11;
    let columns = format!("grid-template-columns: repeat({diameter}, 1fr 1fr 1fr) 1fr;");
    let rows = format!("grid-template-rows: repeat({diameter}, 1fr 1fr);");
    let transition_name = format!("view-transition-name: _{id};");

    rsx!(
        Padded {
            padding: Padding::all(8),
            div {
                class: "hexboard",
                style: "{columns}{rows}{transition_name}",
                onmouseleave: move |_| action(Select::Unconsider),
                {board_html.into_iter()}
            }
        }
    )
}
