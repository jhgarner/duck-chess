use std::collections::HashSet;

use crate::prelude::*;

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<Loc>,
    pub board: Board,
    pub active: Option<Loc>,
    pub targets: HashSet<Loc>,
    pub id: String,
}

pub struct Model;

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut board: Vec<Html> = Vec::new();
        for (down, row) in ctx.props().board.rows().enumerate() {
            for (right, square) in row.iter().enumerate() {
                let at = Loc::new(right, down);

                let color = if (down + right) % 2 == 0 {
                    "background: brown;"
                } else {
                    "background: wheat;"
                };

                let mut classes = vec!["square"];
                if let Some(loc) = ctx.props().active {
                    if loc == at {
                        classes.push("active");
                    }
                }
                if ctx.props().targets.contains(&at) {
                    classes.push("target");
                }

                let onclick = ctx.props().callback.reform(move |_| Loc::new(right, down));

                let id = if let Square::Piece(Color::Black, Piece::Queen) = square {
                    Some("horse")
                } else {
                    None
                };

                board.push(html! {
                    <div class={classes!(classes)} style={color} {onclick}>
                        <crate::piece::Model {id} square={square.clone()}/>
                    </div>
                });
            }
        }

        let size = ctx.props().board.width();

        html! {
            <div class="board" style={format!("grid-template-columns: repeat({size}, 1fr);")}>
                { for board.into_iter() }
            </div>
        }
    }
}
