use crate::dependency_injection::{get_dynamodb_client, get_notifier};
use crate::domain::commands::CreateSessionCommand;
use crate::domain::errors::LogicError;
use crate::notifier::notifier::{ActionType, INotifier, Message};
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::session_table::SessionItem;
use crate::storage::websocket_table::WebsocketItem;
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
            db.write(vec![session.save()?, connection.save()?]).await?;
            session_id
        }
    };

    let message = Message {
        action: ActionType::GetSession,
        data: session_id.clone(),
    };
    notifier.notify(&connection.connection_id, &message).await?;
    Ok(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dependency_injection::get_notifier, test_setup};

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

        // Updates database tables
        let connection = WebsocketItem::from_db(&connection_id, &db).await?;
        let session_id = match connection.session_id {
            Some(session_id) => session_id,
            None => return Err(LogicError::GetItemError("Session not found".to_string())),
        };
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.connection_id, connection_id);
        Ok(())
    }

    #[tokio::test]
    async fn reuses_session_if_it_already_exists() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem {
            connection_id: connection_id.to_string(),
            session_id: Some(session_id.to_string()),
            version: 0,
        };
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = CreateSessionCommand { connection_id };
        let result = handler(&request).await;
        let response = match result {
            Ok(response) => response,
            Err(error) => return Err(error),
        };
        assert_eq!(response, session_id);
        Ok(())
    }
}
