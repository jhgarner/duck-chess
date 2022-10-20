use crate::{auth::Session, prelude::*};

pub async fn connect(config: String) -> Result<Database> {
    let mut client_options = ClientOptions::parse(config).await?;
    client_options.default_database = Some("ducky-cluster".into());
    let client = Client::with_options(client_options)?;
    Ok(client.default_database().unwrap())
}

pub async fn setup_players_database(db: &Database, prefix: &str) -> Result<Collection<Player>> {
    let players: Collection<Player> = db.collection(&format!("{prefix}_Players"));
    players
        .create_index(
            IndexModel::builder()
                .keys(doc! { "name": 1u32 })
                .options(Some(IndexOptions::builder().unique(true).build()))
                .build(),
            None,
        )
        .await?;
    Ok(players)
}

pub async fn setup_games_database(db: &Database, prefix: &str) -> Result<Collection<AnyGame>> {
    let games: Collection<AnyGame> = db.collection(&format!("{prefix}_AllGames"));
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "game.joiner._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "game.maker._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    Ok(games)
}

pub async fn setup_session_database(db: &Database, prefix: &str) -> Result<Collection<Session>> {
    let sessions: Collection<Session> = db.collection(&format!("{prefix}_Sessions"));
    sessions
        .create_index(
            IndexModel::builder()
                .keys(doc! { "player._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    Ok(sessions)
}
