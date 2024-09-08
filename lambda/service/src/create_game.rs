use crate::send_game_state_notification;
use chrono::Utc;
use domain::commands::{CreateGameCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use event_publisher::{self, EventMessage, IEventPublisher};
use notifier::{self, ActionType, INotifier, Message};
use serde_json::json;
use storage::game_table::{GameItem, PlayerItem};
use storage::session_table::{SessionAction, SessionItem};
use storage::IDynamoDbClient;

pub async fn handler(command: &CreateGameCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let event_publisher = event_publisher::get().await;
    let notifier = notifier::get().await;

    let mut session = SessionItem::from_db(&command.session_id, &db).await?;
    let nickname = session.nickname.clone().ok_or(LogicError::NotAllowed)?;
    if session.game_id.is_some() {
        return Ok("Already in game".to_string());
    }

    let game_id = GameItem::create_game_code();
    let mut game = GameItem::new(&game_id, &command.session_id);
    let player = PlayerItem::new(&session.session_id, &session.account_id, &nickname);
    game.players.push(player);
    session.game_id = Some(game_id.clone());
    session.modified_action = SessionAction::JoinGame;
    session.modified_at = Utc::now();
    session.version += 1;

    db.write(vec![game.save()?, session.save()?]).await?;

    println!("Sending new game response");
    let message = Message::new(ActionType::JoinGame(game_id.clone()));
    notifier.notify(&command.connection_id, &message).await?;

    println!("Sending game state notification");
    let command = SendGameStateNotificationCommand {
        game_id: game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    println!("Sending event message");
    let event_message = EventMessage {
        source: "RustLambda-Dev.GameCreated".to_string(),
        detail_type: "Game created".to_string(),
        detail: json!({"game_id": game_id}),
    };
    event_publisher.publish(&event_message).await?;

    Ok(game_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use storage::{game_table::GameAction, websocket_table::WebsocketItem};
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_session_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let request = CreateGameCommand {
            connection_id,
            session_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn errors_if_nickname_not_set() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);
        db.write_single(session.save()?).await?;

        let request = CreateGameCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn creates_new_game() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.nickname = Some("Test".to_string());
        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = CreateGameCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        let game_id = handler(&request).await?;

        // Notifies the connection (1 for new game, 1 for game state)
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 2);

        // Creates game item
        let game = GameItem::from_db(&game_id, &db).await?;
        assert!(game.modified_at > start_time);
        assert_eq!(game.modified_action, GameAction::CreateGame);
        assert_eq!(game.modified_by, session_id);
        assert!(game.mr_eleven.is_none());
        assert_eq!(game.round_finished, false);
        assert_eq!(game.version, 0);

        // Updates session item
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.game_id.unwrap(), game_id);
        assert_eq!(session.modified_action, SessionAction::JoinGame);
        assert!(session.modified_at > start_time);

        // Publishes event
        let event_publisher = event_publisher::get().await;
        let messages = event_publisher.get_messages("RustLambda-Dev.GameCreated");
        assert_eq!(messages.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_if_game_already_exists() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.nickname = Some("Test".to_string());
        session.game_id = Some("ABCD".to_string());
        db.write_single(session.save()?).await?;

        let request = CreateGameCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // Doesn't update database tables
        let session2 = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session2.version, session.version);

        Ok(())
    }
}
