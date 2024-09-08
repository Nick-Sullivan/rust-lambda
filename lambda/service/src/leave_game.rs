use chrono::Utc;
use domain::commands::{LeaveGameCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use game_logic::finish_round;
use storage::game_table::{GameAction, GameItem};
use storage::IDynamoDbClient;

use crate::send_game_state_notification;

pub async fn handler(command: &LeaveGameCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let mut game = GameItem::from_db(&command.game_id, &db).await?;
    game.players.retain(|p| p.player_id != command.session_id);
    game.version += 1;
    game.modified_at = Utc::now();
    game.modified_action = GameAction::LeaveGame;

    if game.players.is_empty() {
        println!("No more players");
        db.write_single(game.delete()?).await?;
        return Ok("Success".to_string());
    }
    if game.players.iter().all(|p| p.finished) {
        println!("All players finished");
        game = finish_round::finish_round(&mut game)?;
    }

    db.write_single(game.save()?).await?;

    let command = SendGameStateNotificationCommand {
        game_id: game.game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use storage::{game_table::PlayerItem, session_table::SessionItem};
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_game_doesnt_exist() {
        test_setup::setup();
        let session_id = Uuid::new_v4().to_string();
        let game_id = Uuid::new_v4().to_string();
        let request = LeaveGameCommand {
            session_id,
            game_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn deletes_game_if_last_player() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let game_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection_id = Uuid::new_v4().to_string();
        let mut game = GameItem::new(&game_id, &session_id);
        game.players
            .push(PlayerItem::new(&session_id, &None, "Player"));
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![session.save()?, game.save()?]).await?;

        let request = LeaveGameCommand {
            game_id: game_id.clone(),
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let game = GameItem::from_db(&game_id, &db).await;
        assert!(game.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn finishes_round_if_all_players_have_finished() -> Result<(), LogicError> {
        test_setup::setup();
        let start_time = Utc::now();
        let db = storage::get().await;

        let game_id = Uuid::new_v4().to_string();
        let session_id1 = Uuid::new_v4().to_string();
        let session_id2 = Uuid::new_v4().to_string();
        let connection_id = Uuid::new_v4().to_string();
        let mut game = GameItem::new(&game_id, &session_id1);
        game.players
            .push(PlayerItem::new(&session_id1, &None, "Player1"));
        game.players.push(PlayerItem {
            player_id: session_id2.clone(),
            account_id: None,
            nickname: "Player2".to_string(),
            win_counter: 0,
            finished: true,
        });
        let session1 = SessionItem::new(&session_id1, &connection_id);
        let session2 = SessionItem::new(&session_id2, &connection_id);
        db.write(vec![session1.save()?, session2.save()?, game.save()?])
            .await?;

        let request = LeaveGameCommand {
            game_id: game_id.clone(),
            session_id: session_id1.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Updates database item
        let game = GameItem::from_db(&game_id, &db).await?;
        assert!(game.round_finished);
        assert_eq!(game.version, 1);
        assert_eq!(game.modified_action, GameAction::LeaveGame);
        assert!(game.modified_at > start_time);

        Ok(())
    }
}
