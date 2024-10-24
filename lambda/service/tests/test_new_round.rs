mod test_setup;

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use chrono::Utc;
    use domain::commands::NewRoundCommand;
    use domain::errors::LogicError;
    use notifier::{self, INotifier};
    use service::new_round::handler;
    use std::vec;
    use storage::game_table::{GameAction, GameItem, PlayerItem};
    use storage::session_table::SessionItem;
    use storage::IDynamoDbClient;
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
