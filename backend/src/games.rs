use mongodb::change_stream::event::OperationType;
use rocket::tokio::select;
use rocket::{
    futures::TryStreamExt,
    response::stream::{Event, EventStream},
    Shutdown,
};
use web_push::{PartialVapidSignatureBuilder, WebPushClient, WebPushMessageBuilder};

use crate::auth::Session;
use crate::prelude::*;

pub async fn get_player_games(player: &Player, games: &Collection<Game>) -> Result<Vec<Game>> {
    let filter = doc! {"$or": [{"white._id": player.id}, {"black._id": player.id}]};
    let player_games = games.find(filter, None).await?.try_collect().await?;
    Ok(player_games)
}

pub async fn get_completed_player_games(
    player: &Player,
    games: &Collection<CompletedGame>,
) -> Result<Vec<CompletedGame>> {
    let filter = doc! {"$or": [{"game.white._id": player.id}, {"game.black._id": player.id}]};
    let player_games = games.find(filter, None).await?.try_collect().await?;
    Ok(player_games)
}

pub async fn get_open_player_games(
    player: &Player,
    open_games: &Collection<GameRequest>,
) -> Result<Vec<GameRequest>> {
    let filter = doc! {"maker._id": player.id};
    let unstarted_games = open_games.find(filter, None).await?.try_collect().await?;
    Ok(unstarted_games)
}

pub async fn get_open_games(open_games: &Collection<GameRequest>) -> Result<Vec<GameRequest>> {
    let open_games = open_games.find(None, None).await?.try_collect().await?;
    Ok(open_games)
}

pub async fn new_open_game(maker: Player, open_games: &Collection<GameRequest>) -> Result<()> {
    let open_game = GameRequest { id: None, maker };

    open_games.insert_one(open_game, None).await?;
    Ok(())
}

pub async fn join_open_game(
    game_id: ObjectId,
    joiner: Player,
    open_games: &Collection<GameRequest>,
    games: &Collection<Game>,
) -> Result<Game> {
    let open_filter = doc! {"_id": game_id};
    let open_game = open_games.find_one_and_delete(open_filter, None).await?;
    if let Some(open_game) = open_game {
        let maker_first: bool = rand::random();
        let (white, black) = if maker_first {
            (open_game.maker, joiner)
        } else {
            (joiner, open_game.maker)
        };
        let mut game = Game {
            id: None,
            board: Board::default(),
            turns: Vec::new(),
            white,
            black,
        };
        let result = games.insert_one(&game, None).await?;
        game.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(game)
    } else {
        bail!("Not a game!")
    }
}

pub struct Notifier {
    pub client: WebPushClient,
    pub crypto: PartialVapidSignatureBuilder,
}

pub async fn apply_turn(
    turn: WithId<Turn>,
    player: Player,
    sessions: &Collection<Session>,
    pusher: &Notifier,
    games: &Collection<Game>,
    completed_games: &Collection<CompletedGame>,
) -> Result<()> {
    let filter = doc! {"_id": turn.id};
    let mut game = games
        .find_one(filter.clone(), None)
        .await?
        .ok_or(anyhow!("Not valid"))?;
    game.apply_turn(&player, *turn)?;

    let other_player = if game.turn() == Color::White {
        game.white.id
    } else {
        game.black.id
    };

    // TODO is it better to set message like this, or make message mutable?
    let message = if let None = game.game_over() {
        games.replace_one(filter, game, None).await?;
        "It's your turn in a Duck Chess game!"
    } else {
        let completed = CompletedGame { id: None, game };
        completed_games.insert_one(&completed, None).await?;
        games.delete_one(filter, None).await?;
        "A Duck Chess game has ended!"
    };

    let filter = doc! { "player._id": other_player.unwrap() };
    let mut subscriptions = sessions.find(filter, None).await?;

    while let Some(session) = subscriptions.try_next().await? {
        if let Some(subscription) = session.subscription {
            let mut sig_builder = pusher.crypto.clone().add_sub_info(&subscription);
            // Firefox refuses the request unless you include an email
            sig_builder.add_claim("sub", "mailto:emailjunk234@gmail.com");
            let sig = sig_builder.build()?;
            let mut builder = WebPushMessageBuilder::new(&subscription)?;
            builder.set_payload(web_push::ContentEncoding::Aes128Gcm, message.as_bytes());
            builder.set_vapid_signature(sig);
            pusher.client.send(builder.build()?).await?;
        }
    }
    Ok(())
}

pub async fn create_game_stream(
    game_id: ObjectId,
    player: Player,
    games: &Collection<Game>,
    mut shutdown: Shutdown,
) -> Result<EventStream![]> {
    let filter = doc! {"_id": game_id};
    let game = games
        .find_one(filter.clone(), None)
        .await?
        .ok_or(anyhow!("No valid game for id"))?;
    if !game.player(&player).is_empty() {
        let matcher = doc! {"$match": {"documentKey._id": game_id}};
        let mut change_stream = games.watch([matcher], None).await?;

        // TODO split this up into a function or something so it's a little less bad
        Ok(EventStream! {
            loop {
                select! {
                    change = change_stream.try_next() => {
                        if let Ok(Some(change)) = change {
                            if let OperationType::Replace = change.operation_type {
                                let game = change.full_document.unwrap();
                                yield Event::json(&game);
                            }
                        } else {
                            break;
                        }
                    }
                    _ = &mut shutdown => break
                }
            }
        })
    } else {
        bail!("No valid game")
    }
}
