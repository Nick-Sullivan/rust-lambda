use crate::dependency_injection::get_dynamodb_client;
use crate::domain::commands::DestroyConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::websocket_table::WebsocketItem;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;

    let transaction = WebsocketItem::get(&command.connection_id)?;
    let db_lock = db.lock().await;
    let response = db_lock.read(transaction).await?;
    let attribute = response
        .item
        .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
    let item = WebsocketItem::from_map(&attribute)?;
    let transaction = item
        .delete()
        .map_err(|e| LogicError::DeleteItemError(e.to_string()))?;
    db_lock.write(vec![transaction]).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::websocket_table::WebsocketItem, test_setup};
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
    async fn destroys_connection() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let item = WebsocketItem::new(&connection_id);
        let transaction = item.save().unwrap();

        let db = get_dynamodb_client().await;
        let db_lock = db.lock().await;
        let _ = db_lock.write(vec![transaction]).await;
        drop(db_lock);

        let request = DestroyConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }
}
