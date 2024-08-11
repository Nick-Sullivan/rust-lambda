use crate::dependency_injection::get_dynamodb_client;
use crate::domain::commands::CreateConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::websocket_table::WebsocketItem;

pub async fn handler(command: &CreateConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;

    let item = WebsocketItem::new(&command.connection_id);
    let transaction = item.save()?;
    let db_lock = db.lock().await;
    db_lock.write(vec![transaction]).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use uuid::Uuid;

    #[tokio::test]
    async fn creates_new_connection() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[tokio::test]
    async fn errors_if_connection_already_exists() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let item = WebsocketItem::new(&connection_id);
        let transaction = item.save().unwrap();
        let db = get_dynamodb_client().await;
        let db_lock = db.lock().await;
        let _ = db_lock.write(vec![transaction]).await;
        drop(db_lock);

        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
    }
}
