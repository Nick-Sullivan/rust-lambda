mod test_setup;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::{commands::CreateConnectionCommand, errors::LogicError};
    use service::create_connection::handler;
    use storage::{websocket_table::WebsocketItem, IDynamoDbClient};
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
