use std::collections::HashMap;

use dioxus::{
    core::ScopeId,
    hooks::{use_future, use_signal},
    signals::{GlobalSignal, ReadableExt, ReadableVecExt, Signal, WritableExt},
};

use crate::{
    board::{AnyGame, Square, SquareId},
    style::{clear_style, set_style},
    transition::transition_callback,
};

fn add_game(game: &AnyGame) {
    if let Some(prev) = GAMES.write().games.insert(
        game.id.unwrap().to_string(),
        Signal::new_in_scope(game.clone(), ScopeId::APP),
    ) {
        prev.manually_drop();
    };
}

pub fn use_all_games() -> Signal<Vec<Signal<AnyGame>>> {
    let mut result = use_signal(|| GAMES.resolve().read().games.values().copied().collect());
    use_future(move || async move {
        if result.is_empty() {
            load_games().await;
            result.set(GAMES.resolve().read().games.values().copied().collect());
        }
    });
    result
}

pub fn use_game(id: String) -> Signal<Option<Signal<AnyGame>>> {
    let mut result = use_signal(|| GAMES.resolve().read().games.get(&id).copied());
    use_future(move || {
        let id = id.clone();
        async move {
            if let Some(game) = GAMES.resolve().read().games.get(&id) {
                listen_game(id.clone(), *game).await;
            } else {
                load_games().await;
                let game = *GAMES.resolve().read().games.get(&id).unwrap();
                result.set(Some(game));
                listen_game(id.clone(), game).await;
            }
        }
    });
    result
}

async fn load_games() {
    let games = crate::rpc::fetch_games().await.unwrap_or_default();
    for any_game in games {
        add_game(&any_game);
    }
}

async fn listen_game(id: String, mut holder: Signal<AnyGame>) {
    if let Ok(mut stream) = crate::rpc::game_events(id).await {
        let mut locations = HashMap::new();
        while let Some(Ok(value)) = stream.next().await {
            let mut style = String::new();
            for (loc, square) in value.game.pieces() {
                let id = match square {
                    Square::Piece(_, _, SquareId(id)) => format!("_{id}"),
                    Square::Duck => "duck".to_string(),
                    Square::Empty => continue,
                };
                if let Some(old_loc) = locations.insert(square, loc)
                    && old_loc != loc
                {
                    style.push_str(&format!(
                        "#{id} {{ view-transition-name: {id}; view-transition-class: bulk_piece; }}"
                    ));
                }
            }
            set_style("game_pushed", style);
            transition_callback(move || {
                holder.set(value);
            })
            .on_complete_callback(|| {
                clear_style("game_pushed");
            });
        }
    }
}

static GAMES: GlobalSignal<AllGames> = Signal::global(AllGames::default);

#[derive(Default, Debug, Clone)]
struct AllGames {
    games: HashMap<String, Signal<AnyGame>>,
}
