mod auth;
mod games;
mod mongo;
mod prelude;

use auth::*;
use games::*;
use mongo::*;
use prelude::*;
use rocket::{http::CookieJar, response::stream::EventStream, serde::json::Json, Shutdown, State};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

type Response<T> = RawResponse<Json<T>>;
type RawResponse<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

#[post("/login", data = "<player>")]
async fn login(
    player: Json<PasswordPlayer>,
    cookies: &CookieJar<'_>,
    players: &State<Collection<Player>>,
) -> Response<Player> {
    let player = login_user(players, player.name.clone(), player.password.clone()).await?;
    let cookie = mk_session_cookie(player.clone());
    cookies.add_private(cookie);
    Ok(Json(player))
}

#[post("/signup", data = "<player>")]
async fn signup(
    player: Json<PasswordPlayer>,
    cookies: &CookieJar<'_>,
    players: &State<Collection<Player>>,
) -> Response<Player> {
    let player = new_user(players, player.name.clone(), player.password.clone()).await?;
    let cookie = mk_session_cookie(player.clone());
    cookies.add(cookie);
    Ok(Json(player))
}

#[get("/games")]
async fn in_games(
    session: Session,
    games: &State<Collection<Game>>,
    open_games: &State<Collection<GameRequest>>,
    completed_games: &State<Collection<CompletedGame>>,
) -> Response<MyGames> {
    let player = session.player;
    let started = get_player_games(&player, games).await?;
    let unstarted = get_open_player_games(&player, open_games).await?;
    let completed = get_completed_player_games(&player, &completed_games).await?;
    let my_games = MyGames {
        started,
        unstarted,
        completed,
    };
    Ok(Json(my_games))
}

#[get("/open_games")]
async fn open_games(
    _session: Session,
    games: &State<Collection<GameRequest>>,
) -> Response<Vec<GameRequest>> {
    let open_games = get_open_games(games).await?;
    Ok(Json(open_games))
}

#[post("/new_game")]
async fn new_game(session: Session, games: &State<Collection<GameRequest>>) -> Response<()> {
    new_open_game(session.player, games).await?;
    Ok(Json(()))
}

#[post("/join_game", data = "<game_id>")]
async fn join_game(
    game_id: Json<ObjectId>,
    session: Session,
    open_games: &State<Collection<GameRequest>>,
    games: &State<Collection<Game>>,
) -> Response<Game> {
    let game = join_open_game(*game_id, session.player, open_games, games).await?;
    Ok(Json(game))
}

#[post("/turn", data = "<turn>")]
async fn submit_turn(
    turn: Json<WithId<Turn>>,
    session: Session,
    games: &State<Collection<Game>>,
    completed_games: &State<Collection<CompletedGame>>,
) -> Response<()> {
    apply_turn(turn.0, session.player, games, completed_games).await?;
    Ok(Json(()))
}

#[post("/poll", data = "<game_id>")]
async fn poll(
    game_id: Json<ObjectId>,
    session: Session,
    games: &State<Collection<Game>>,
    shutdown: Shutdown,
) -> RawResponse<EventStream![]> {
    let stream = create_game_stream(*game_id, session.player, games, shutdown).await?;
    Ok(stream)
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config: String = figment.extract_inner("mongo").expect("config");
    let db = connect(config).await?;
    let players = setup_players_database(&db).await?;
    let games = setup_games_database(&db).await?;
    let open_games = setup_open_games_database(&db).await?;
    let completed_games = setup_completed_games_database(&db).await?;
    let _rocket = rocket
        .manage(players)
        .manage(games)
        .manage(open_games)
        .manage(completed_games)
        .mount(
            "/",
            routes![
                index,
                login,
                signup,
                in_games,
                open_games,
                new_game,
                join_game,
                submit_turn,
                poll
            ],
        )
        .launch()
        .await?;
    Ok(())
}
