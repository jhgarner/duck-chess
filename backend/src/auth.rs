use std::time::SystemTime;

use rocket::{request::{FromRequest, self}, Request, http::{Cookie, Status}, outcome::Outcome};

use crate::prelude::*;

const TOKEN: &'static str = "token";

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub time: u64,
    pub player: Player
}

#[derive(Debug)]
pub enum SessionError {
    Missing,
    BadSession(serde_json::Error),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = req.cookies().get_private(TOKEN) {
            let result = serde_json::from_str(cookie.value());
            match result {
                Ok(session) => Outcome::Success(session),
                Err(error) => Outcome::Failure((Status::BadRequest, SessionError::BadSession(error))),
            }
        } else {
            Outcome::Failure((Status::BadRequest, SessionError::Missing))
        }
    }
}

pub async fn login_user(players: &Collection<Player>, name: String, real_password: String) -> Result<Player> {
    let players = players.clone_with_type::<PasswordPlayer>();
    if let Some(found_player) = players.find_one(Some(doc! {"name": &name}), None).await? {
        let hasher = HashBuilder::from_phc(&found_player.password).map_err(hash_fail_reason)?;
        if hasher.is_valid(&real_password) {
            let player = Player {
                id: found_player.id,
                name
            };
            Ok(player)
        } else {
            bail!("incorrect")
        }
    } else {
        bail!("incorrect")
    }
}

pub async fn new_user(players: &Collection<Player>, name: String, real_password: String) -> Result<Player> {
    name_unique(players, &name).await?;
    let players = players.clone_with_type::<PasswordPlayer>();
    let hasher = HashBuilder::new_std(libreauth::pass::PasswordStorageStandard::Nist80063b)
        .finalize()
        .map_err(hash_fail_reason)?;
    let hashed_password = hasher.hash(&real_password).map_err(hash_fail_reason)?;
    let new_player = Player {
        id: None,
        name: name.clone(),
    };
    let with_password = PasswordPlayer {
        password: hashed_password,
        player: new_player
    };
    let result = players.insert_one(with_password, None).await?;
    let player = Player {
        id: result.inserted_id.as_object_id(),
        name,
    };
    Ok(player)
}

async fn name_unique(players: &Collection<Player>, name: &String) -> Result<()> {
    if let Some(_) = players.find_one(doc! { "name": name }, None).await? {
        bail!("Name already taken")
    } else {
        Ok(())
    }
}

fn hash_fail_reason(err: ErrorCode) -> Error {
    match err {
        ErrorCode::PasswordTooShort => anyhow!("Password too short"),
        ErrorCode::PasswordTooLong => anyhow!("Password too long"),
        _ => anyhow!("Something went wrong"),
    }
}

pub fn mk_session_cookie(player: Player) -> Cookie<'static> {
    let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let session = Session { time, player };
    Cookie::build(TOKEN, serde_json::to_string(&session).unwrap())
        .secure(true)
        .http_only(true)
        .finish()
}
