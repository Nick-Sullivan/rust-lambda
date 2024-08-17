use crate::dependency_injection::get_dynamodb_client;
use crate::domain::commands::DestroyConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::websocket_table::WebsocketItem;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;
    let connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    db.write(vec![connection.delete()?]).await?;
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
    async fn destroys_connection() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;

        let connection_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        db.write_single(connection.save()?).await?;

        let request = DestroyConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());
        Ok(())
    }
}
