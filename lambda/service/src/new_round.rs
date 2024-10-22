use crate::send_game_state_notification;
use chrono::Utc;
use domain::commands::{NewRoundCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use storage::game_table::{GameAction, GameItem};
use storage::session_table::SessionItem;
use storage::IDynamoDbClient;

pub async fn handler(command: &NewRoundCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let session = SessionItem::from_db(&command.session_id, &db).await?;

    let game_id = if let Some(game_id) = session.game_id {
        game_id
    } else {
        println!("No game");
        return Ok("No game".to_string());
    };

    let mut game = GameItem::from_db(&game_id, &db).await?;
    if !game.round_finished {
        println!("Round not finished");
        return Ok("Round not finished".to_string());
    }

    game.round_finished = false;
    game.version += 1;
    // game.round_id += 1;
    game.modified_action = GameAction::NewRound;
    game.modified_by = command.session_id.clone();
    game.modified_at = Utc::now();
    for player in game.players.iter_mut() {
        player.finished = false;
        // player.outcome = ;
        // player.rolls = Vec::new();
    }
    db.write_single(game.save()?).await?;

    println!("Sending game state notification");
    let command = SendGameStateNotificationCommand {
        game_id: game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    Ok(game_id)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::test_setup;
    use notifier::{self, INotifier};
    use storage::game_table::{GameAction, PlayerItem};
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_session_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let request = NewRoundCommand {
            connection_id,
            session_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn does_nothing_if_game_doesnt_exist() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);
        db.write_single(session.save()?).await?;

        let request = NewRoundCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // No notifications
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_if_game_isnt_finished() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let game_id = Uuid::new_v4().to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.game_id = Some(game_id.clone());
        let mut game = GameItem::new(&game_id, &session_id);
        game.round_finished = false;
        db.write(vec![session.save()?, game.save()?]).await?;

        let request = NewRoundCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // No notifications
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn creates_new_round() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let game_id = Uuid::new_v4().to_string();
        let nickname = "Test".to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.game_id = Some(game_id.clone());
        session.nickname = Some(nickname.clone());
        let mut game = GameItem::new(&game_id, &session_id);
        let player = PlayerItem::new(&session.session_id, &session.account_id, &nickname);
        game.players.push(player);
        game.round_finished = true;
        db.write(vec![session.save()?, game.save()?]).await?;

        let request = NewRoundCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Updates game item
        let game = GameItem::from_db(&game_id, &db).await?;
        assert!(game.modified_at > start_time);
        assert_eq!(game.modified_action, GameAction::NewRound);
        assert_eq!(game.modified_by, session_id);
        assert_eq!(game.round_finished, false);
        assert_eq!(game.version, 1);

        Ok(())
    }
}
