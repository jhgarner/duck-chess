use crate::common::game::GameTypes;
use futures::TryStreamExt;
use mongodb::change_stream::{
    ChangeStream,
    event::{ChangeStreamEvent, OperationType},
};
use web_push::{VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder};

use super::{
    prelude::*,
    state::{Notifier, SessionRecord},
};

pub async fn get_player_games(
    player: &Player,
    games: &Collection<AnyGame>,
) -> Result<Vec<AnyGame>> {
    let filter = doc! {"$or": [{"game.maker._id": player.id}, {"game.joiner._id": player.id}]};
    Ok(games.find(filter).await?.try_collect().await?)
}

pub async fn get_open_games(games: &Collection<AnyGame>) -> Result<Vec<WithId<GameRequest>>> {
    let open_games: Vec<AnyGame> = games
        .find(doc! {"game.type": "Request"})
        .await?
        .try_collect()
        .await?;
    Ok(open_games
        .into_iter()
        .map(|game| match game.game {
            GameOrRequest::Request(request) => WithId::new(game.id.unwrap(), request),
            _ => panic!("request filter returned a non-request game"),
        })
        .collect())
}

pub async fn new_open_game(maker: Player, games: &Collection<AnyGame>) -> Result<ObjectId> {
    let open_game = AnyGame {
        id: None,
        game: GameOrRequest::Request(GameRequest {
            maker,
            game_type: GameTypes::Square,
        }),
    };

    Ok(games
        .insert_one(open_game)
        .await?
        .inserted_id
        .as_object_id()
        .unwrap())
}

pub async fn join_open_game(
    game_id: ObjectId,
    joiner: Player,
    games: &Collection<AnyGame>,
    sessions: &Collection<SessionRecord>,
    notifier: &Notifier,
) -> Result<()> {
    let filter = doc! {"_id": game_id};
    let open_game = games.find_one(filter.clone()).await?;
    if let Some(AnyGame {
        id,
        game: GameOrRequest::Request(request),
    }) = open_game
    {
        let maker_color = if rand::random() {
            Color::White
        } else {
            Color::Black
        };
        let maker_id = request.maker.id.unwrap();
        let joiner_id = joiner.id.unwrap();
        let game = request
            .game_type
            .mk_game(request.maker, joiner, maker_color);
        games
            .replace_one(
                filter,
                AnyGame {
                    id,
                    game: GameOrRequest::Game(game),
                },
            )
            .await?;
        send_notification(maker_id, "Duck Chess game started!", sessions, notifier).await?;
        send_notification(joiner_id, "Duck Chess game started!", sessions, notifier).await?;
        Ok(())
    } else {
        bail!("Not a game!")
    }
}

pub async fn apply_turn(
    turn: WithId<SomeTurn>,
    player: Player,
    sessions: &Collection<SessionRecord>,
    notifier: &Notifier,
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

        let message = if game.game_over().is_none() {
            games
                .replace_one(
                    filter,
                    AnyGame {
                        id: with_id.id,
                        game: GameOrRequest::Game(game),
                    },
                )
                .await?;
            "It's your turn in a Duck Chess game!"
        } else {
            games
                .replace_one(
                    filter,
                    AnyGame {
                        id: with_id.id,
                        game: GameOrRequest::Completed(game),
                    },
                )
                .await?;
            "A Duck Chess game has ended!"
        };

        send_notification(other_player, message, sessions, notifier).await?;
        Ok(())
    } else {
        bail!("Invalid game!")
    }
}

pub async fn create_change_stream(
    game_id: ObjectId,
    player: Player,
    games: &Collection<AnyGame>,
) -> Result<(AnyGame, ChangeStream<ChangeStreamEvent<AnyGame>>)> {
    let filter = doc! {"_id": game_id};
    let with_id = games
        .find_one(filter)
        .await?
        .ok_or_else(|| anyhow!("No valid game for id"))?;
    if !with_id.game.in_game(&player) {
        bail!("No valid game")
    }

    let matcher = doc! {"$match": {"documentKey._id": game_id, "operationType": "replace"}};
    let change_stream = games.watch().pipeline([matcher]).await?;
    Ok((with_id, change_stream))
}

pub async fn next_game_update(
    player: &Player,
    change_stream: &mut ChangeStream<ChangeStreamEvent<AnyGame>>,
) -> Result<Option<AnyGame>> {
    while let Some(change) = change_stream.try_next().await? {
        if let OperationType::Replace = change.operation_type {
            let game = change.full_document.unwrap();
            if game.game.in_game(player) {
                return Ok(Some(game));
            }
            return Ok(None);
        }
    }
    Ok(None)
}

async fn send_notification(
    player: ObjectId,
    message: &str,
    sessions: &Collection<SessionRecord>,
    notifier: &Notifier,
) -> Result<()> {
    let mut subscriptions = sessions.find(doc! { "player._id": player }).await?;

    while let Some(session) = subscriptions.try_next().await? {
        if let Some(subscription) = session.subscription {
            let mut sig_builder: VapidSignatureBuilder<'_> =
                notifier.crypto.clone().add_sub_info(&subscription);
            sig_builder.add_claim("sub", "mailto:emailjunk234@gmail.com");
            let sig = sig_builder.build()?;
            let mut builder = WebPushMessageBuilder::new(&subscription);
            builder.set_payload(web_push::ContentEncoding::Aes128Gcm, message.as_bytes());
            builder.set_vapid_signature(sig);
            notifier.client.send(builder.build()?).await?;
        }
    }

    Ok(())
}
