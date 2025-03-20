use common::game::GameTypes;
use mongodb::change_stream::event::OperationType;
use rocket::tokio::select;
use rocket::{
    Shutdown,
    futures::TryStreamExt,
    response::stream::{Event, EventStream},
};
use web_push::{
    IsahcWebPushClient, PartialVapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::auth::Session;
use crate::prelude::*;

pub async fn get_player_games(
    player: &Player,
    games: &Collection<AnyGame>,
) -> Result<Vec<AnyGame>> {
    let filter = doc! {"$or": [{"game.maker._id": player.id}, {"game.joiner._id": player.id}]};
    let player_games = games.find(filter).await?.try_collect().await?;
    Ok(player_games)
}

pub async fn get_open_games(games: &Collection<AnyGame>) -> Result<Vec<WithId<GameRequest>>> {
    let filter = doc! {"game.type": "Request"};
    let open_games: Vec<AnyGame> = games.find(filter).await?.try_collect().await?;
    let open_games = open_games
        .into_iter()
        .map(|game| {
            if let GameOrRequest::Request(request) = game.game {
                WithId::new(game.id.unwrap(), request)
            } else {
                panic!("Filter didn't work!")
            }
        })
        .collect();
    Ok(open_games)
}

pub async fn new_open_game(maker: Player, open_games: &Collection<AnyGame>) -> Result<ObjectId> {
    let game = GameOrRequest::Request(GameRequest {
        maker,
        game_type: GameTypes::Hex,
    });
    let open_game = AnyGame { id: None, game };

    let id = open_games
        .insert_one(open_game)
        .await?
        .inserted_id
        .as_object_id()
        .unwrap();
    Ok(id)
}

pub async fn join_open_game(
    game_id: ObjectId,
    joiner: Player,
    games: &Collection<AnyGame>,
    sessions: &Collection<Session>,
    pusher: &Notifier,
) -> Result<()> {
    let filter = doc! {"_id": game_id};
    let open_game = games.find_one(filter.clone()).await?;
    if let Some(AnyGame {
        id,
        game: GameOrRequest::Request(request),
    }) = open_game
    {
        let maker_first: bool = rand::random();
        let maker_color = if maker_first {
            Color::White
        } else {
            Color::Black
        };
        let maker_id = request.maker.id.unwrap();
        let joiner_id = joiner.id.unwrap();
        let game = request
            .game_type
            .mk_game(request.maker, joiner, maker_color);
        let with_id = AnyGame {
            id,
            game: GameOrRequest::Game(game),
        };
        games.replace_one(filter, with_id).await?;
        send_notification(maker_id, "Duck Chess game started!", sessions, pusher).await?;
        send_notification(joiner_id, "Duck Chess game started!", sessions, pusher).await?;
        Ok(())
    } else {
        bail!("Not a game!")
    }
}

pub struct Notifier {
    pub client: IsahcWebPushClient,
    pub crypto: PartialVapidSignatureBuilder,
}

pub async fn apply_turn(
    turn: WithId<SomeTurn>,
    player: Player,
    sessions: &Collection<Session>,
    pusher: &Notifier,
    games: &Collection<AnyGame>,
) -> Result<()> {
    let filter = doc! {"_id": turn.id};
    let with_id = games
        .find_one(filter.clone())
        .await?
        .ok_or_else(|| anyhow!("Not valid"))?;
    if let GameOrRequest::Game(mut game) = with_id.game {
        game.apply_turn(&player, *turn)?;

        let other_player = if game.turn() == game.maker_color {
            game.maker.id.unwrap()
        } else {
            game.joiner.id.unwrap()
        };

        let message;

        if game.game_over().is_none() {
            let new_game = AnyGame {
                id: with_id.id,
                game: GameOrRequest::Game(game),
            };
            games.replace_one(filter, new_game).await?;
            message = "It's your turn in a Duck Chess game!";
        } else {
            let completed = AnyGame {
                id: with_id.id,
                game: GameOrRequest::Completed(game),
            };
            games.replace_one(filter, completed).await?;
            message = "A Duck Chess game has ended!";
        };

        send_notification(other_player, message, sessions, pusher).await?;
        Ok(())
    } else {
        bail!("Invalid game!")
    }
}

async fn send_notification(
    player: ObjectId,
    message: &str,
    sessions: &Collection<Session>,
    pusher: &Notifier,
) -> Result<()> {
    let filter = doc! { "player._id": player };
    let mut subscriptions = sessions.find(filter).await?;

    while let Some(session) = subscriptions.try_next().await? {
        if let Some(subscription) = session.subscription {
            let mut sig_builder = pusher.crypto.clone().add_sub_info(&subscription);
            // Firefox refuses the request unless you include an email
            sig_builder.add_claim("sub", "mailto:emailjunk234@gmail.com");
            let sig = sig_builder.build()?;
            let mut builder = WebPushMessageBuilder::new(&subscription);
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
    games: &Collection<AnyGame>,
    mut shutdown: Shutdown,
) -> Result<EventStream![]> {
    let filter = doc! {"_id": game_id};
    let with_id = games
        .find_one(filter.clone())
        .await?
        .ok_or_else(|| anyhow!("No valid game for id"))?;
    if with_id.game.in_game(&player) {
        let matcher = doc! {"$match": {"documentKey._id": game_id}};
        let mut change_stream = games.watch().pipeline([matcher]).await?;

        // TODO split this up into a function or something so it's a little less bad
        Ok(EventStream! {
            yield Event::json(&with_id);
            loop {
                select! {
                    change = change_stream.try_next() => {
                        if let Ok(Some(change)) = change {
                            if let OperationType::Replace = change.operation_type {
                                let game = change.full_document.unwrap();
                                if game.game.in_game(&player) {
                                    yield Event::json(&game);
                                } else {
                                    break;
                                }
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
