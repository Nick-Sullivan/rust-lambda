use chrono::Utc;
use domain::commands::DestroyConnectionCommand;
use domain::errors::LogicError;
use storage::dependency_injection::get_dynamodb_client;
use storage::dynamodb_client::IDynamoDbClient;
use storage::session_table::{SessionAction, SessionItem};
use storage::websocket_table::WebsocketItem;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;
    let connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    match &connection.session_id {
        Some(session_id) => {
            let mut session = SessionItem::from_db(&session_id, &db).await?;
            session.modified_action = SessionAction::PendingTimeout;
            session.modified_at = Utc::now();
            session.version += 1;
            db.write(vec![connection.delete()?, session.save()?])
                .await?;
        }
        None => {
            db.write_single(connection.delete()?).await?;
        }
    }
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_connection_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let request = DestroyConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn destroys_connection() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;

        let connection_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        db.write_single(connection.save()?).await?;

        let request = DestroyConnectionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let connection = WebsocketItem::from_db(&connection_id, &db).await;
        assert!(connection.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn disconnects_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new_with_session(&connection_id, &session_id);
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = DestroyConnectionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes connection from database
        let connection = WebsocketItem::from_db(&connection_id, &db).await;
        assert!(connection.is_err());

        // Updates session database
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.modified_action, SessionAction::PendingTimeout);
        assert!(session.modified_at > start_time);
        Ok(())
    }
}
