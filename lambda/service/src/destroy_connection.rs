use domain::commands::DestroyConnectionCommand;
use domain::errors::LogicError;
use storage::dependency_injection::get_dynamodb_client;
use storage::dynamodb_client::IDynamoDbClient;
use storage::websocket_table::WebsocketItem;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;
    let connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    db.write(vec![connection.delete()?]).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use storage::websocket_table::WebsocketItem;
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
