use std::time::SystemTime;

use axum_extra::extract::cookie::Cookie;
use dioxus::logger::tracing;

use super::{prelude::*, state::SessionRecord};

pub const TOKEN_COOKIE: &str = "token";

pub async fn login_user(
    players: &Collection<Player>,
    name: String,
    real_password: String,
) -> Result<Player> {
    let players = players.clone_with_type::<PasswordPlayer>();
    tracing::warn!("{:?}", &players);
    if let Some(found_player) = players.find_one(doc! {"name": &name}).await? {
        let hasher = HashBuilder::from_phc(&found_player.password)?;
        if hasher.is_valid(&real_password) {
            Ok(Player {
                id: found_player.id,
                name,
            })
        } else {
            bail!("incorrect")
        }
    } else {
        bail!("incorrect")
    }
}

pub async fn new_user(
    players: &Collection<Player>,
    name: String,
    real_password: String,
) -> Result<Player> {
    name_unique(players, &name).await?;
    let players = players.clone_with_type::<PasswordPlayer>();
    let hasher =
        HashBuilder::new_std(libreauth::pass::PasswordStorageStandard::NoStandard).finalize()?;
    let hashed_password = hasher.hash(&real_password)?;
    let with_password = PasswordPlayer {
        password: hashed_password,
        player: Player {
            id: None,
            name: name.clone(),
        },
    };
    let result = players.insert_one(with_password).await?;
    Ok(Player {
        id: result.inserted_id.as_object_id(),
        name,
    })
}

pub async fn create_session_cookie(
    player: Player,
    sessions: &Collection<SessionRecord>,
) -> Result<Cookie<'static>> {
    let session = SessionRecord {
        id: None,
        subscription: None,
        time: SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs(),
        player,
    };
    let result = sessions.insert_one(session).await?;

    Ok(Cookie::build((
        TOKEN_COOKIE,
        serde_json::to_string(&result.inserted_id).unwrap(),
    ))
    .secure(true)
    .http_only(true)
    .path("/")
    .build())
}

pub async fn update_session(
    subscription: web_push::SubscriptionInfo,
    session: SessionRecord,
    sessions: &Collection<SessionRecord>,
) -> Result<()> {
    let id = session.id.unwrap();
    let updated = SessionRecord {
        subscription: Some(subscription),
        ..session
    };
    sessions.replace_one(doc! { "_id": id }, updated).await?;
    Ok(())
}

pub async fn clear_player_sessions(
    session: &SessionRecord,
    sessions: &Collection<SessionRecord>,
) -> Result<()> {
    sessions
        .delete_many(doc! { "player._id": session.player.id })
        .await?;
    Ok(())
}

async fn name_unique(players: &Collection<Player>, name: &String) -> Result<()> {
    if players.find_one(doc! { "name": name }).await?.is_some() {
        bail!("Name already taken")
    } else {
        Ok(())
    }
}

pub fn removal_cookie() -> Cookie<'static> {
    Cookie::build((TOKEN_COOKIE, ""))
        .path("/")
        .http_only(true)
        .build()
}
