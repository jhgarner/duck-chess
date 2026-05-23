use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::board::AnyGame;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameEvents {
    FullState {
        your_games: HashMap<String, AnyGame>,
    },
    NewState {
        id: String,
        game: AnyGame,
    },
}
