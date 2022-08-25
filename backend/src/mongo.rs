use crate::{prelude::*, auth::Session};

pub async fn connect(config: String) -> Result<Database> {
    let mut client_options =
        ClientOptions::parse(config).await?;
    client_options.default_database = Some("ducky-cluster".into());
    let client = Client::with_options(client_options)?;
    Ok(client.default_database().unwrap())
}

pub async fn setup_players_database(db: &Database) -> Result<Collection<Player>> {
    let players: Collection<Player> = db.collection("Players");
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

pub async fn setup_games_database(db: &Database) -> Result<Collection<Game>> {
    let games: Collection<Game> = db.collection("Games");
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

pub async fn setup_open_games_database(db: &Database) -> Result<Collection<GameRequest>> {
    let games: Collection<GameRequest> = db.collection("OpenGames");
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

pub async fn setup_session_database(db: &Database) -> Result<Collection<Session>> {
    let games: Collection<Session> = db.collection("Sessions");
    games
        .create_index(
            IndexModel::builder()
                .keys(doc! { "player._id": 1u32 })
                .build(),
            None,
        )
        .await?;
    Ok(games)
}

pub async fn setup_completed_games_database(db: &Database) -> Result<Collection<CompletedGame>> {
    // TODO this is almost equivalent to the games db
    let games: Collection<CompletedGame> = db.collection("CompletedGames");
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
