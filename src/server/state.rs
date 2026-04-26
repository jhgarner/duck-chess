use axum::{Extension, Router, extract::FromRequestParts, http::request::Parts};
use tower_cookies::Cookies;
use web_push::{IsahcWebPushClient, PartialVapidSignatureBuilder, VapidSignatureBuilder};

use super::{config::ServerConfig, mongo, prelude::*};

pub type DB<T> = Extension<Collection<T>>;

pub const TOKEN_COOKIE: &str = "token";

#[derive(Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub struct SessionRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Option<ObjectId>,
    pub time: u64,
    pub subscription: Option<web_push::SubscriptionInfo>,
    pub player: Player,
}

impl<T: Send + Sync + 'static> FromRequestParts<T> for SessionRecord {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &T,
    ) -> std::result::Result<Self, Self::Rejection> {
        <Self as axum::extract::OptionalFromRequestParts<T>>::from_request_parts(parts, state)
            .await?
            .ok_or(unauthorized())
    }
}

impl<T: Send + Sync + 'static> axum::extract::OptionalFromRequestParts<T> for SessionRecord {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &T,
    ) -> std::result::Result<Option<Self>, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await.unwrap();
        let sessions =
            <Extension<Collection<_>> as axum::extract::FromRequestParts<T>>::from_request_parts(
                parts, state,
            )
            .await?;
        Ok(if let Some(cookie) = cookies.get(TOKEN_COOKIE) {
            let id: ObjectId = serde_json::from_str(cookie.value())?;
            sessions.find_one(doc! { "_id": id }).await?
        } else {
            None
        })
    }
}

#[derive(Clone)]
pub struct Notifier {
    pub client: IsahcWebPushClient,
    pub crypto: PartialVapidSignatureBuilder,
}

pub async fn build_state(router: Router) -> Result<Router> {
    let config = ServerConfig::from_env()?;
    let db = mongo::connect(config.mongo_url.clone()).await?;
    let players = mongo::setup_players_database(&db, &config.prefix).await?;
    let games = mongo::setup_games_database(&db, &config.prefix).await?;
    let sessions = mongo::setup_session_database(&db, &config.prefix).await?;
    let notifier = Notifier {
        client: IsahcWebPushClient::new()?,
        crypto: VapidSignatureBuilder::from_pem_no_sub(config.pem.as_bytes())?,
    };

    Ok(router
        .layer(Extension(players))
        .layer(Extension(games))
        .layer(Extension(sessions))
        .layer(Extension(notifier)))
}
