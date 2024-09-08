use domain::commands::CreateConnectionCommand;
use domain::errors::LogicError;
use storage::{self, websocket_table::WebsocketItem, IDynamoDbClient};

pub async fn handler(command: &CreateConnectionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let connection = WebsocketItem::new(&command.connection_id);
    db.write_single(connection.save()?).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn creates_new_connection() -> Result<(), LogicError> {
        test_setup::setup();
        let start_time = Utc::now();
        let connection_id = Uuid::new_v4().to_string();
        let request = CreateConnectionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        let db = storage::get().await;
        let item = WebsocketItem::from_db(&connection_id, &db).await?;
        assert_eq!(item.connection_id, connection_id);
        assert!(item.modified_at > start_time);
        Ok(())
    }

    #[tokio::test]
    async fn errors_if_connection_already_exists() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let item = WebsocketItem::new(&connection_id);
        db.write_single(item.save()?).await?;

        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
        Ok(())
    }
}
