mod test_setup;

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use chrono::Utc;
    use domain::commands::SetSessionCommand;
    use domain::errors::LogicError;
    use notifier::{self, INotifier};
    use service::set_session::handler;
    use storage::session_table::{SessionAction, SessionItem};
    use storage::websocket_table::WebsocketItem;
    use storage::IDynamoDbClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_connection_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let request = SetSessionCommand {
            connection_id,
            session_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn updates_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let old_connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &old_connection_id);

        let connection_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);

        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = SetSessionCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;

        // Returns OK
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Updates connection table
        let connection = WebsocketItem::from_db(&connection_id, &db).await?;
        assert!(connection.modified_at > start_time);
        let session_id = match connection.session_id {
            Some(session_id) => session_id,
            None => return Err(LogicError::GetItemError("Session not found".to_string())),
        };

        // Updates session table
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.connection_id, connection_id);
        assert_eq!(session.modified_action, SessionAction::Reconnected);
        assert!(connection.modified_at > start_time);
        Ok(())
    }
}
