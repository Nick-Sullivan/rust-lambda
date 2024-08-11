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
    let db_lock = db.lock().await;

    let transaction = WebsocketItem::get(&command.connection_id)?;
    let output = db_lock.read(transaction).await?;
    let attribute = output
        .item
        .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
    let mut connection = WebsocketItem::from_map(&attribute)?;

    let session_id = match connection.session_id {
        Some(session_id) => session_id,
        None => {
            let session_id = Uuid::new_v4().to_string();
            let item = SessionItem::new(&session_id, &command.connection_id);
            let transaction = item.save()?;
            connection.session_id = Some(session_id.clone());
            connection.version += 1;
            let connection_transaction = connection.save()?;
            let transactions = vec![transaction, connection_transaction];
            db_lock.write(transactions).await?;
            session_id
        }
    };

    let message = Message {
        action: ActionType::GetSession,
        data: session_id.clone(),
    };
    let notifier = get_notifier().await;
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
        let connection_id = Uuid::new_v4().to_string();
        let item = WebsocketItem::new(&connection_id);
        let transaction = item.save()?;
        let db = get_dynamodb_client().await;
        let db_lock = db.lock().await;
        let _ = db_lock.write(vec![transaction]).await;
        drop(db_lock);

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
        let db_lock = db.lock().await;
        let connection = WebsocketItem::get(&connection_id)?;
        let connection = db_lock
            .read(connection)
            .await?
            .item
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
        let connection = WebsocketItem::from_map(&connection)?;
        assert!(connection.session_id.is_some());

        let transaction = SessionItem::get(&connection.session_id.unwrap())?;
        let output = db_lock.read(transaction).await?;
        let attribute = output
            .item
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
        let session = SessionItem::from_map(&attribute)?;
        assert_eq!(session.connection_id, connection_id);
        Ok(())
    }

    #[tokio::test]
    async fn reuses_session_if_it_already_exists() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem {
            connection_id: connection_id.to_string(),
            session_id: Some(session_id.to_string()),
            version: 0,
        }
        .save()
        .unwrap();
        let session = SessionItem::new(&session_id, &connection_id)
            .save()
            .unwrap();
        let db = get_dynamodb_client().await;
        let db_lock = db.lock().await;
        let _ = db_lock.write(vec![connection, session]).await;
        drop(db_lock);

        let request = CreateSessionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());
        assert_eq!(result.unwrap(), session_id);
        // TODO check it sends notifications
        // TODO check it created the item in two tables
    }
}
