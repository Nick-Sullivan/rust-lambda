use chrono::Utc;
use domain::commands::CreateSessionCommand;
use domain::errors::LogicError;
use notifier::dependency_injection::get_notifier;
use notifier::notifier::{ActionType, INotifier, Message};
use serde_json::json;
use storage::dependency_injection::get_dynamodb_client;
use storage::dynamodb_client::IDynamoDbClient;
use storage::session_table::SessionItem;
use storage::websocket_table::WebsocketItem;
use uuid::Uuid;

pub async fn handler(command: &CreateSessionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;
    let notifier = get_notifier().await;

    let mut connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    let session_id = match connection.session_id {
        Some(session_id) => session_id,
        None => {
            let session_id = Uuid::new_v4().to_string();
            let session = SessionItem::new(&session_id, &command.connection_id);
            connection.session_id = Some(session_id.clone());
            connection.version += 1;
            connection.modified_at = Utc::now();
            db.write(vec![session.save()?, connection.save()?]).await?;
            session_id
        }
    };

    let message = Message::new(ActionType::GetSession, json!(session_id.clone()));
    notifier.notify(&connection.connection_id, &message).await?;
    Ok(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use notifier::dependency_injection::get_notifier;
    use storage::session_table::SessionAction;

    #[tokio::test]
    async fn errors_if_connection_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let request = CreateSessionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn creates_new_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        db.write_single(connection.save()?).await?;

        let request = CreateSessionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;

        // Returns OK
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Notifies the connection
        let notifier = get_notifier().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Updates connection item
        let connection = WebsocketItem::from_db(&connection_id, &db).await?;
        assert!(connection.modified_at > start_time);
        let session_id = match connection.session_id {
            Some(session_id) => session_id,
            None => return Err(LogicError::GetItemError("Session not found".to_string())),
        };

        // Updates session item
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.connection_id, connection_id);
        assert!(session.game_id.is_none());
        assert_eq!(session.modified_action, SessionAction::CreateConnection);
        assert!(session.modified_at > start_time);
        Ok(())
    }

    #[tokio::test]
    async fn reuses_session_if_it_already_exists() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new_with_session(&connection_id, &session_id);
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = CreateSessionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await?;
        assert_eq!(result, session_id);

        // Doesn't update database tables
        let connection2 = WebsocketItem::from_db(&connection_id, &db).await?;
        assert_eq!(connection2.version, connection.version);
        let session2 = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session2.version, session.version);

        Ok(())
    }
}
