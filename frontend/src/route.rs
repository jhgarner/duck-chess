use crate::ingame::InGame;
use crate::mainmenu::MainMenu;
use crate::newgame::NewGame;
use crate::prelude::*;

// #[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    MainMenu {},
    #[route("/ui/game/:id")]
    InGame { id: String },
    #[route("/ui/newgame")]
    NewGame {},
}
