use serde::{Deserialize, Serialize};

use crate::Square;

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuBoard {
    pub grid: Vec<Square>,
}

impl MenuBoard {
    pub fn new(pieces: Vec<Square>) -> Self {
        MenuBoard { grid: pieces }
    }
}
