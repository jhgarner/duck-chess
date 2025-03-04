use std::time::SystemTime;

use libreauth::pass::PasswordStorageStandard;
use rocket::{
    Request, State,
    http::Cookie,
    outcome::Outcome,
    request::{self, FromRequest},
};
use web_push::SubscriptionInfo;

use crate::prelude::*;

const TOKEN: &str = "token";

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub time: u64,
    pub subscription: Option<SubscriptionInfo>,
    pub player: Player,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = req.cookies().get_private(TOKEN) {
            let result: Result<ObjectId, _> = serde_json::from_str(cookie.value());
            match result {
                Ok(id) => {
                    let filter = doc! { "_id": id };
                    let collection: &State<Collection<Session>> = req.guard().await.unwrap();
                    if let Some(session) = collection.find_one(filter).await.unwrap() {
                        Outcome::Success(session)
                    } else {
                        Outcome::Forward(())
                    }
                }
                Err(_) => Outcome::Forward(()),
            }
        } else {
            Outcome::Forward(())
        }
    }
}

pub async fn login_user(
    players: &Collection<Player>,
    name: String,
    real_password: String,
) -> Result<Player> {
    let players = players.clone_with_type::<PasswordPlayer>();
    if let Some(found_player) = players.find_one(doc! {"name": &name}).await? {
        let hasher = HashBuilder::from_phc(&found_player.password)?;
        if hasher.is_valid(&real_password) {
            let player = Player {
                id: found_player.id,
                name,
            };
            Ok(player)
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
    let hasher = HashBuilder::new_std(PasswordStorageStandard::NoStandard).finalize()?;
    let hashed_password = hasher.hash(&real_password)?;
    let new_player = Player {
        id: None,
        name: name.clone(),
    };
    let with_password = PasswordPlayer {
        password: hashed_password,
        player: new_player,
    };
    let result = players.insert_one(with_password).await?;
    let player = Player {
        id: result.inserted_id.as_object_id(),
        name,
    };
    Ok(player)
}

async fn name_unique(players: &Collection<Player>, name: &String) -> Result<()> {
    if players.find_one(doc! { "name": name }).await?.is_some() {
        bail!("Name already taken")
    } else {
        Ok(())
    }
}

pub async fn mk_session_cookie(
    player: Player,
    sessions: &Collection<Session>,
) -> Result<Cookie<'static>> {
    let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let session = Session {
        id: None,
        subscription: None,
        time,
        player,
    };
    let result = sessions.insert_one(session).await?;

    let cookie = Cookie::build(TOKEN, serde_json::to_string(&result.inserted_id).unwrap())
        .secure(true)
        .http_only(true)
        .finish();
    Ok(cookie)
}

pub async fn update_session(
    subscription: SubscriptionInfo,
    session: Session,
    sessions: &Collection<Session>,
) -> Result<()> {
    let id = session.id.unwrap();
    let filter = doc! { "_id": id };
    let session = Session {
        subscription: Some(subscription),
        ..session
    };
    sessions.replace_one(filter, session).await?;
    Ok(())
}

pub async fn clear_my_sessions(session: Session, sessions: &Collection<Session>) -> Result<()> {
    let filter = doc! { "player._id": session.player.id };
    sessions.delete_many(filter).await?;
    Ok(())
}
