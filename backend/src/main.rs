mod auth;
mod games;
mod mongo;
mod prelude;

use std::path::Path;

use auth::*;
use games::*;
use mongo::*;
use prelude::*;
use rocket::{
    fs::{FileServer, NamedFile},
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

#[get("/session_notifications")]
async fn session_notifications(session: Session) -> Response<bool> {
    Ok(Json(session.subscription.is_some()))
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
async fn in_games(session: Session, games: &State<Collection<AnyGame>>) -> Response<Vec<AnyGame>> {
    let player = session.player;
    let player_games = get_player_games(&player, games).await?;
    Ok(Json(player_games))
}

#[get("/open_games")]
async fn open_games(
    _session: Session,
    games: &State<Collection<AnyGame>>,
) -> Response<Vec<WithId<GameRequest>>> {
    let open_games = get_open_games(games).await?;
    Ok(Json(open_games))
}

#[post("/new_game")]
async fn new_game(session: Session, games: &State<Collection<AnyGame>>) -> Response<ObjectId> {
    let id = new_open_game(session.player, games).await?;
    Ok(Json(id))
}

#[post("/join_game/<game_id>")]
async fn join_game(
    game_id: &str,
    session: Session,
    games: &State<Collection<AnyGame>>,
    sessions: &State<Collection<Session>>,
    notifier: &State<Notifier>,
) -> Response<()> {
    join_open_game(
        game_id.parse().unwrap(),
        session.player,
        games,
        sessions,
        notifier,
    )
    .await?;
    Ok(Json(()))
}

#[post("/turn", data = "<turn>")]
async fn submit_turn(
    turn: Json<WithId<Turn>>,
    session: Session,
    games: &State<Collection<AnyGame>>,
    sessions: &State<Collection<Session>>,
    notifier: &State<Notifier>,
) -> Response<()> {
    apply_turn(turn.0, session.player, sessions, notifier, games).await?;
    Ok(Json(()))
}

#[get("/poll/<game_id>")]
async fn poll(
    game_id: &str,
    session: Session,
    games: &State<Collection<AnyGame>>,
    shutdown: Shutdown,
) -> RawResponse<EventStream![]> {
    let game_id = ObjectId::parse_str(game_id).unwrap();
    let stream = create_game_stream(game_id, session.player, games, shutdown).await?;
    Ok(stream)
}

#[get("/<_..>")]
async fn frontend(frontend_dist: &State<String>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&format!("{frontend_dist}/index.html")))
        .await
        .ok()
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
    let sessions = setup_session_database(&db, &prefix).await?;
    let pem: String = figment.extract_inner("pem").unwrap();
    let notifier = Notifier {
        client: WebPushClient::new()?,
        crypto: VapidSignatureBuilder::from_pem_no_sub(pem.as_bytes())?,
    };
    let _rocket = rocket
        .manage(players)
        .manage(games)
        .manage(sessions)
        .manage(notifier)
        .manage(frontend_dist.clone())
        .mount("/", FileServer::from(frontend_dist))
        .mount("/ui", routes![frontend])
        .mount(
            "/api/",
            routes![
                login,
                session_ok,
                session_bad,
                session_notifications,
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
