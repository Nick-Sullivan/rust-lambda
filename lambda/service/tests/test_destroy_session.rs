mod test_setup;

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use domain::commands::DestroySessionCommand;
    use domain::errors::LogicError;
    use notifier::{self, INotifier};
    use service::destroy_session::handler;
    use storage::session_table::SessionItem;
    use storage::websocket_table::WebsocketItem;
    use storage::IDynamoDbClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_session_doesnt_exist() {
        test_setup::setup();
        let session_id = Uuid::new_v4().to_string();
        let request = DestroySessionCommand {
            connection_id: None,
            session_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn destroys_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![session.save()?, connection.save()?]).await?;

        let request = DestroySessionCommand {
            session_id: session_id.clone(),
            connection_id: Some(connection_id.clone()),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let session = SessionItem::from_db(&session_id, &db).await;
        assert!(session.is_err());

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn destroys_session_if_no_connection() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);
        db.write_single(session.save()?).await?;

        let request = DestroySessionCommand {
            session_id: session_id.clone(),
            connection_id: None,
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let session = SessionItem::from_db(&session_id, &db).await;
        assert!(session.is_err());

        // No notification
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 0);

        Ok(())
    }
}
