mod auth;
mod games;
mod mongo;
mod prelude;

use auth::*;
use games::*;
use mongo::*;
use prelude::*;
use rocket::{
    fs::FileServer,
    http::CookieJar,
    response::stream::EventStream,
    serde::json::Json,
    Shutdown, State,
};
use web_push::{SubscriptionInfo, VapidSignatureBuilder, WebPushClient};

#[macro_use]
extern crate rocket;

type Response<T> = RawResponse<Json<T>>;
type RawResponse<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

#[post("/login", data = "<player>")]
async fn login(
    player: Json<PasswordPlayer>,
    cookies: &CookieJar<'_>,
    sessions: &State<Collection<Session>>,
    players: &State<Collection<Player>>,
) -> Response<Player> {
    let player = login_user(players, player.name.clone(), player.password.clone()).await?;
    let cookie = mk_session_cookie(player.clone(), sessions).await?;
    cookies.add_private(cookie);
    Ok(Json(player))
}

#[post("/session")]
async fn session_ok(session: Session) -> Response<Option<Player>> {
    Ok(Json(Some(session.player)))
}

#[post("/session", rank = 2)]
async fn session_bad() -> Response<Option<Player>> {
    Ok(Json(None))
}

#[post("/signup", data = "<player>")]
async fn signup(
    player: Json<PasswordPlayer>,
    cookies: &CookieJar<'_>,
    sessions: &State<Collection<Session>>,
    players: &State<Collection<Player>>,
) -> Response<Player> {
    let player = new_user(players, player.name.clone(), player.password.clone()).await?;
    let cookie = mk_session_cookie(player.clone(), sessions).await?;
    cookies.add_private(cookie);
    Ok(Json(player))
}

#[post("/logout")]
async fn logout(session: Session, sessions: &State<Collection<Session>>) -> Response<()> {
    clear_my_sessions(session, sessions).await?;
    Ok(Json(()))
}

#[post("/subscribe", data = "<subscription>")]
async fn subscribe(
    subscription: Json<SubscriptionInfo>,
    session: Session,
    sessions: &State<Collection<Session>>,
) -> Response<()> {
    update_session(subscription.0, session, sessions).await?;
    Ok(Json(()))
}

#[get("/public_key")]
async fn public_key(notifier: &State<Notifier>) -> RawResponse<String> {
    let result = base64_url::encode(&notifier.crypto.get_public_key());
    Ok(result)
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
    let (my_turn, other_turn) = started
        .into_iter()
        .partition(|game| game.is_player_turn(&player));
    let unstarted = get_open_player_games(&player, open_games).await?;
    let completed = get_completed_player_games(&player, completed_games).await?;
    let completed = completed.into_iter().map(|game| game.game).collect();
    let my_games = MyGames {
        my_turn,
        other_turn,
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
    sessions: &State<Collection<Session>>,
    notifier: &State<Notifier>,
    completed_games: &State<Collection<CompletedGame>>,
) -> Response<()> {
    apply_turn(
        turn.0,
        session.player,
        sessions,
        notifier,
        games,
        completed_games,
    )
    .await?;
    Ok(Json(()))
}

#[get("/poll/<game_id>")]
async fn poll(
    game_id: &str,
    session: Session,
    games: &State<Collection<Game>>,
    shutdown: Shutdown,
) -> RawResponse<EventStream![]> {
    let game_id = ObjectId::parse_str(game_id).unwrap();
    let stream = create_game_stream(game_id, session.player, games, shutdown).await?;
    Ok(stream)
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let mongo_url: String = figment.extract_inner("mongo").unwrap();
    let frontend_dist: String = figment.extract_inner("frontend").unwrap();
    let prefix: String = figment.extract_inner("prefix").unwrap();
    let db = connect(mongo_url).await?;
    let players = setup_players_database(&db, &prefix).await?;
    let games = setup_games_database(&db, &prefix).await?;
    let open_games = setup_open_games_database(&db, &prefix).await?;
    let completed_games = setup_completed_games_database(&db, &prefix).await?;
    let sessions = setup_session_database(&db, &prefix).await?;
    let pem: String = figment.extract_inner("pem").unwrap();
    let notifier = Notifier {
        client: WebPushClient::new()?,
        crypto: VapidSignatureBuilder::from_pem_no_sub(pem.as_bytes())?,
    };
    let _rocket = rocket
        .manage(players)
        .manage(games)
        .manage(open_games)
        .manage(completed_games)
        .manage(sessions)
        .manage(notifier)
        .mount("/", FileServer::from(frontend_dist))
        .mount(
            "/",
            routes![
                login,
                session_ok,
                session_bad,
                signup,
                logout,
                subscribe,
                public_key,
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
