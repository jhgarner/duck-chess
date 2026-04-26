use dioxus::prelude::*;

use crate::prelude::*;

#[cfg(feature = "server")]
use crate::server::state::{DB, Notifier, SessionRecord};

#[post("/rpc/session", session: Option<SessionRecord>)]
pub async fn fetch_session() -> ServerFnResult<Option<Player>> {
    Ok(session.map(|session| session.player))
}

#[get("/rpc/games", session: SessionRecord, games: DB<AnyGame>)]
pub async fn fetch_games() -> ServerFnResult<Vec<AnyGame>> {
    Ok(crate::server::games::get_player_games(&session.player, &games).await?)
}

#[get("/rpc/games/open", _: SessionRecord, games: DB<AnyGame>)]
pub async fn fetch_open_games() -> ServerFnResult<Vec<WithId<GameRequest>>> {
    Ok(crate::server::games::get_open_games(&games).await?)
}

#[post("/rpc/games/new", session: SessionRecord, games: DB<AnyGame>)]
pub async fn create_game() -> ServerFnResult<ObjectId> {
    Ok(crate::server::games::new_open_game(session.player, &games).await?)
}

#[post("/rpc/games/join", session: SessionRecord, games: DB<AnyGame>, sessions: DB<SessionRecord>, notifier: Extension<Notifier>)]
pub async fn join_game_rpc(game_id: String) -> ServerFnResult<()> {
    let game_id =
        bson::oid::ObjectId::parse_str(game_id).map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 400,
            details: None,
        })?;
    crate::server::games::join_open_game(game_id, session.player, &games, &sessions, &notifier)
        .await
        .map_err(ServerFnError::from)
}

#[post("/rpc/games/turn", session: SessionRecord, games: DB<AnyGame>, sessions: DB<SessionRecord>, notifier: Extension<Notifier>)]
pub async fn submit_turn_rpc(turn: WithId<SomeTurn>) -> ServerFnResult<()> {
    crate::server::games::apply_turn(turn, session.player, &sessions, &notifier, &games)
        .await
        .map_err(ServerFnError::from)
}

#[get("/rpc/notifications/enabled", session: SessionRecord)]
pub async fn notifications_enabled() -> ServerFnResult<bool> {
    Ok(session.subscription.is_some())
}

#[get("/rpc/notifications/public-key", notifier: Extension<Notifier>)]
pub async fn public_key_rpc() -> ServerFnResult<String> {
    Ok(base64_url::encode(&notifier.crypto.get_public_key()))
}

#[post("/rpc/notifications/subscribe", session: SessionRecord, sessions: DB<SessionRecord>)]
pub async fn subscribe_rpc(subscription_json: String) -> ServerFnResult<()> {
    let subscription = serde_json::from_str(&subscription_json).map_err(ServerFnError::from)?;
    update_session(subscription, session, &sessions)
        .await
        .map_err(ServerFnError::from)
}

#[post("/rpc/signup", jar: Cookies, players: DB<Player>, sessions: DB<SessionRecord>)]
pub async fn signup(player: PasswordPlayer) -> Result<Player> {
    let player = new_user(&players, player.name.clone(), player.password.clone()).await?;
    let cookie = create_session_cookie(player.clone(), &sessions).await?;
    jar.add(cookie);
    Ok(player)
}

#[post("/rpc/login", jar: Cookies, players: DB<Player>, sessions: DB<SessionRecord>)]
pub async fn login(player: PasswordPlayer) -> dioxus::prelude::Result<Player> {
    let player = login_user(&players, player.name.clone(), player.password.clone()).await?;
    let cookie = create_session_cookie(player.clone(), &sessions).await?;
    jar.add(cookie);
    Ok(player)
}

#[post("/rpc/logout", session: SessionRecord, jar: Cookies, sessions: DB<SessionRecord>)]
pub async fn logout() -> Result<()> {
    clear_player_sessions(&session, &sessions).await?;
    jar.remove(removal_cookie());
    Ok(())
}

#[post("/rpc/game_events", session: SessionRecord, games: DB<AnyGame>)]
pub async fn game_events(game_id: String) -> Result<JsonStream<AnyGame>> {
    use async_stream::stream;

    let game_id = ObjectId::parse_str(game_id)?;
    let (initial, mut change_stream) =
        crate::server::games::create_change_stream(game_id, session.player.clone(), &games).await?;

    let event_stream = stream! {
        yield initial;
        loop {
            match crate::server::games::next_game_update(&session.player, &mut change_stream).await {
                Ok(Some(game)) => yield game,
                Ok(None) => {
                    break;
                }
                Err(error) => {
                    log::error!("game event stream failed: {error:?}");
                    break;
                }
            }
        }
    };

    Ok(JsonStream::new(event_stream))
}
