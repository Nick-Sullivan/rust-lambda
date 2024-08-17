use crate::dependency_injection::get_dynamodb_client;
use crate::domain::commands::CreateConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::websocket_table::WebsocketItem;

pub async fn handler(command: &CreateConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;
    let connection = WebsocketItem::new(&command.connection_id);
    db.write_single(connection.save()?).await?;
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
    async fn errors_if_connection_already_exists() -> Result<(), LogicError> {
        test_setup::setup();
        let db = get_dynamodb_client().await;

        let connection_id = Uuid::new_v4().to_string();
        let item = WebsocketItem::new(&connection_id);
        db.write_single(item.save()?).await?;

        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
        Ok(())
    }
}
