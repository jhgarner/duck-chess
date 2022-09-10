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

pub async fn setup_games_database(db: &Database, prefix: &str) -> Result<Collection<Game>> {
    let games: Collection<Game> = db.collection(&format!("{prefix}_Games"));
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "black._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "white._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    Ok(games)
}

pub async fn setup_open_games_database(db: &Database, prefix: &str) -> Result<Collection<GameRequest>> {
    let games: Collection<GameRequest> = db.collection(&format!("{prefix}_OpenGames"));
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "maker._id": 1u32 })
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

pub async fn setup_completed_games_database(db: &Database, prefix: &str) -> Result<Collection<CompletedGame>> {
    // TODO this is almost equivalent to the games db
    let games: Collection<CompletedGame> = db.collection(&format!("{prefix}_CompletedGames"));
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "game.black._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "game.white._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    Ok(games)
}
