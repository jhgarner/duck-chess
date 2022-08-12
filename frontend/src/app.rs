use std::{ops::Deref, sync::Arc};

use crate::{board::BoardDrawer, request::*};
use common::{
    rules::{apply, apply_duck, mk_board, valid_duck, valid_locations},
    *,
};
use egui::{mutex::Mutex, Ui, Vec2};
use serde::de::IgnoredAny;

enum AuthState {
    LoggedOut(PasswordPlayer),
    Loading,
    LoggedIn(Player, State),
}

impl Default for AuthState {
    fn default() -> Self {
        let player = Player {
            id: None,
            name: "me".into(),
        };
        let with_password = PasswordPlayer {
            password: "123456789".into(),
            player,
        };
        AuthState::LoggedOut(with_password)
    }
}

enum State {
    Loading,
    MainMenu(MyGames),
    NewGame(Option<Vec<GameRequest>>),
    InGame(InGame),
}

#[derive(Clone)]
pub struct MessageChannel {
    message: Arc<Mutex<Option<Message>>>,
    ctx: egui::Context,
}

impl MessageChannel {
    fn new(ctx: egui::Context) -> Self {
        MessageChannel {
            message: Arc::default(),
            ctx,
        }
    }
    pub fn write(&self, message: Message) {
        *self.message.lock() = Some(message);
        self.ctx.request_repaint();
    }

    pub fn extract(&self) -> Option<Message> {
        std::mem::take(&mut self.message.lock())
    }
}

pub struct DuckApp {
    state: AuthState,
    extras: Extras,
}

impl Deref for DuckApp {
    type Target = Extras;
    fn deref(&self) -> &Self::Target {
        &self.extras
    }
}

pub struct Extras {
    message: MessageChannel,
    client: Client,
    board_drawer: BoardDrawer,
}

pub enum Message {
    Empty,
    LoggingIn,
    LoggedIn(Player),
    GotGames(MyGames),
    NewGame,
    GotOpenGames(Vec<GameRequest>),
    InGame(Game),
    SpaceClicked(Loc),
    GameUpdate(Game),
    Back,
}

pub struct InGame {
    game_state: GameState,
    poller: EventStream,
    game: Game,
}

#[derive(Debug, Hash, Clone)]
pub enum GameState {
    New(),
    Clicked(Loc, Vec<Action>),
    PlaceDuck(Game, Loc, Action),
    Promote(Game, [[Square; 4]; 1], Loc, Action),
}

impl DuckApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let message = MessageChannel::new(cc.egui_ctx.clone());
        let client = Client::new(message.clone());
        let board_drawer = BoardDrawer::new(cc);

        Self {
            state: AuthState::default(),
            extras: Extras {
                message,
                client,
                board_drawer,
            },
        }
    }

    fn handle(&mut self, message: Message) {
        match message {
            Message::Empty => {}
            Message::LoggingIn => {
                self.state = AuthState::Loading;
            }
            Message::LoggedIn(player) => {
                self.state = AuthState::LoggedIn(player, State::Loading);
                self.client
                    .get("games", |games: MyGames| Message::GotGames(games));
            }
            Message::GotGames(games) => {
                if let AuthState::LoggedIn(_, state) = &mut self.state {
                    *state = State::MainMenu(games);
                }
            }
            Message::NewGame => {
                if let AuthState::LoggedIn(_, state) = &mut self.state {
                    *state = State::NewGame(None);
                    self.client.get("open_games", |games: Vec<GameRequest>| {
                        Message::GotOpenGames(games)
                    });
                }
            }
            Message::GotOpenGames(games) => {
                if let AuthState::LoggedIn(_, State::NewGame(old_games)) = &mut self.state {
                    *old_games = Some(games);
                }
            }
            Message::InGame(game) => {
                if let AuthState::LoggedIn(_, state @ State::MainMenu(_)) = &mut self.state {
                    let poller = self
                        .extras
                        .client
                        .poll("poll", &game.id.unwrap(), |turn| Message::GameUpdate(turn));
                    let in_game = InGame {
                        game_state: GameState::New(),
                        poller,
                        game,
                    };
                    *state = State::InGame(in_game);
                }
            }
            Message::SpaceClicked(loc) => {
                if let AuthState::LoggedIn(
                    _,
                    State::InGame(InGame {
                        game_state, game, ..
                    }),
                ) = &mut self.state
                {
                    match game_state {
                        GameState::New() => {
                            let actions = valid_locations(&game, loc);
                            *game_state = GameState::Clicked(loc, actions);
                        }
                        GameState::Clicked(from, actions) => {
                            if let Some(action) = actions
                                .iter()
                                .find(|action| action.get_target(&game).from(*from) == loc)
                            {
                                let backup_game = game.clone();
                                apply(game, *from, *action);
                                if let Action::Promote(_, _) = action {
                                    *game_state = GameState::Promote(
                                        backup_game,
                                        mk_board(game),
                                        *from,
                                        *action,
                                    );
                                } else {
                                    *game_state = GameState::PlaceDuck(backup_game, *from, *action);
                                }
                            } else {
                                let actions = valid_locations(&game, loc);
                                *game_state = GameState::Clicked(loc, actions);
                            }
                        }
                        GameState::PlaceDuck(og_game, from, action) => {
                            if valid_duck(&game, loc) {
                                apply_duck(game, loc);
                                let turn = Turn {
                                    from: *from,
                                    action: *action,
                                    duck_to: loc,
                                };
                                game.turns.push(turn);
                                // let message = self.message.clone();
                                let wrapped_turn = WithId::new(game.id.unwrap(), turn);
                                self.extras
                                    .client
                                    .post("turn", &wrapped_turn, empty_message);
                                *game_state = GameState::New();
                            } else {
                                // There's probably a way to avoid cloning og_game...
                                *game_state = GameState::PlaceDuck(og_game.clone(), *from, *action);
                            }
                        }
                        GameState::Promote(og_game, promo_board, from, action) => {
                            if let Action::Promote(_, piece) = action {
                                *piece = promo_board[0][loc.right];
                                // This should be doable without cloning too...
                                *game_state = GameState::PlaceDuck(
                                    og_game.clone(),
                                    from.clone(),
                                    action.clone(),
                                );
                            }
                        }
                    }
                }
            }
            Message::GameUpdate(new_game) => {
                if let AuthState::LoggedIn(_, State::InGame(InGame { game, .. })) = &mut self.state
                {
                    *game = new_game;
                }
            }
            Message::Back => match &mut self.state {
                AuthState::LoggedOut(player) => self.state = AuthState::LoggedOut(player.clone()),
                AuthState::Loading => self.state = AuthState::default(),
                AuthState::LoggedIn(player, state) => match state {
                    State::Loading => self.state = AuthState::default(),
                    State::MainMenu(_) => self.state = AuthState::default(),
                    State::NewGame(_) => {
                        self.extras.message.write(Message::LoggedIn(player.clone()))
                    }
                    State::InGame(in_game) => match &mut in_game.game_state {
                        GameState::New() => {
                            in_game.poller.cancel();
                            self.extras.message.write(Message::LoggedIn(player.clone()));
                        }
                        GameState::Clicked(_, _) => in_game.game_state = GameState::New(),
                        GameState::PlaceDuck(game, _, _) => {
                            in_game.game = game.clone();
                            in_game.game_state = GameState::New();
                        }
                        GameState::Promote(game, _, _, _) => {
                            in_game.game = game.clone();
                            in_game.game_state = GameState::New();
                        }
                    },
                },
            },
        }
    }
}
impl Extras {
    fn draw_login_page(&self, player: &mut PasswordPlayer, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Duck Chess Login");

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut player.name);
            });
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.text_edit_singleline(&mut player.password);
            });

            if ui.button("Login").clicked() {
                self.message.write(Message::LoggingIn);
                self.client
                    .post("login", &player, |player| Message::LoggedIn(player));
            }

            if ui.button("Signup").clicked() {
                self.message.write(Message::LoggingIn);
                self.client
                    .post("signup", &player, |player| Message::LoggedIn(player));
            }
        });
    }
}

impl DuckApp {
    fn draw(&mut self, ui: &mut Ui) {
        match &mut self.state {
            AuthState::LoggedOut(player) => {
                self.extras.draw_login_page(player, ui);
            }
            AuthState::Loading => {
                ui.heading("Logging in or signing up...");
            }
            AuthState::LoggedIn(player, state) => match state {
                State::Loading => {
                    ui.heading("Loading Games...");
                }
                State::MainMenu(games) => {
                    if ui.button("Logout").clicked() {
                        self.extras.message.write(Message::Back)
                    }
                    if ui.button("New Game").clicked() {
                        self.extras.message.write(Message::NewGame);
                    }

                    let (your_turn, other_turn): (Vec<_>, Vec<_>) = games
                        .started
                        .iter()
                        .partition(|game| game.player(player).contains(&game.turn()));
                    ui.heading("Your Turn:");
                    ui.vertical_centered(|ui| {
                        for game in your_turn {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Game between {} and {}",
                                    game.white.name, game.black.name
                                ));
                                ui.allocate_ui(Vec2::new(50.0, 50.0), |ui| {
                                    let throwaway = &MessageChannel::new(ui.ctx().clone());
                                    let board = self.extras.board_drawer.layout(
                                        ui,
                                        &game.board.0,
                                        throwaway,
                                    );
                                    for cell in board.iter() {
                                        cell.draw(ui);
                                    }
                                    if throwaway.extract().is_some() {
                                        self.extras.message.write(Message::InGame(game.clone()));
                                    }
                                });
                            });
                        }
                    });

                    ui.heading("Other's Turn:");
                    ui.vertical_centered(|ui| {
                        for game in other_turn {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Game between {} and {}",
                                    game.white.name, game.black.name
                                ));
                                ui.allocate_ui(Vec2::new(50.0, 50.0), |ui| {
                                    let throwaway = &MessageChannel::new(ui.ctx().clone());
                                    let board = self.extras.board_drawer.layout(
                                        ui,
                                        &game.board.0,
                                        throwaway,
                                    );
                                    for cell in board.iter() {
                                        cell.draw(ui);
                                    }
                                    if throwaway.extract().is_some() {
                                        self.extras.message.write(Message::InGame(game.clone()));
                                    }
                                });
                            });
                        }
                    });

                    ui.heading("Completed Turn:");
                    ui.vertical_centered(|ui| {
                        for game in &games.completed {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Game between {} and {}",
                                    game.game.white.name, game.game.black.name
                                ));
                                ui.allocate_ui(Vec2::new(50.0, 50.0), |ui| {
                                    let throwaway = &MessageChannel::new(ui.ctx().clone());
                                    let board = self.extras.board_drawer.layout(
                                        ui,
                                        &game.game.board.0,
                                        throwaway,
                                    );
                                    for cell in board.iter() {
                                        cell.draw(ui);
                                    }
                                    if throwaway.extract().is_some() {
                                        self.extras
                                            .message
                                            .write(Message::InGame(game.game.clone()));
                                    }
                                });
                            });
                        }
                    });

                    ui.heading("Games that haven't started:");
                    ui.vertical_centered(|ui| {
                        for _game in &games.unstarted {
                            ui.horizontal(|ui| {
                                ui.label("Game: ");
                                if ui.button("Cancel (doesn't work)").clicked() {
                                    println!("Cancel not implemented");
                                }
                            });
                        }
                    });
                }
                State::NewGame(games) => {
                    if ui.button("Back").clicked() {
                        self.extras.message.write(Message::Back)
                    }
                    ui.heading("New Game");

                    if ui.button("Create New Game").clicked() {
                        let player = player.clone();
                        self.extras
                            .client
                            .post("new_game", &(), move |_: IgnoredAny| {
                                Message::LoggedIn(player)
                            });
                    }

                    if let Some(games) = games {
                        ui.vertical_centered(|ui| {
                            for game in games {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Game by: {}: ", game.maker.name));
                                    if ui.button("Join").clicked() {
                                        self.extras.client.post(
                                            "join_game",
                                            &game.id,
                                            |game: Game| Message::InGame(game),
                                        );
                                    }
                                });
                            }
                        });
                    } else {
                        ui.heading("Loading open games...");
                    }
                }
                State::InGame(in_game) => {
                    if ui.button("Back").clicked() {
                        self.extras.message.write(Message::Back)
                    }
                    if let GameState::Promote(_, board, _, _) = &in_game.game_state {
                        let grid =
                            self.extras
                                .board_drawer
                                .layout(ui, &board, &self.extras.message);
                        for cell in grid.iter() {
                            cell.draw(ui);
                        }
                    } else {
                        let mut grid = self.extras.board_drawer.layout(
                            ui,
                            &in_game.game.board.0,
                            &self.extras.message,
                        );
                        match &in_game.game_state {
                            GameState::New() => {}
                            GameState::PlaceDuck(_, _, _) => {
                                for cell in grid.iter_mut() {
                                    if let Square::Empty = cell.square {
                                        cell.actionable = true;
                                    }
                                }
                            }
                            GameState::Clicked(loc, actions) => {
                                grid.get_mut(*loc).active = true;
                                for action in actions {
                                    let target = action.get_target(&in_game.game).from(*loc);
                                    grid.get_mut(target).actionable = true;
                                }
                            }
                            GameState::Promote(_, _, _, _) => panic!("Unreachable!"),
                        }
                        for cell in grid.iter() {
                            cell.draw(ui);
                        }
                    }
                }
            },
        };
    }
}

impl eframe::App for DuckApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(message) = self.message.extract() {
            self.handle(message);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw(ui);
        });
    }
}

fn empty_message(_: IgnoredAny) -> Message {
    Message::Empty
}
