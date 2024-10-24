mod test_setup;

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use chrono::Utc;
    use domain::{commands::DestroyConnectionCommand, errors::LogicError};
    use event_publisher::IEventPublisher;
    use service::destroy_connection::handler;
    use storage::{
        session_table::{SessionAction, SessionItem},
        websocket_table::WebsocketItem,
        IDynamoDbClient,
    };
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
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new(&connection_id);
        db.write_single(connection.save()?).await?;

        let request = DestroyConnectionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let connection = WebsocketItem::from_db(&connection_id, &db).await;
        assert!(connection.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn disconnects_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let connection = WebsocketItem::new_with_session(&connection_id, &session_id);
        let session = SessionItem::new(&session_id, &connection_id);
        db.write(vec![connection.save()?, session.save()?]).await?;

        let request = DestroyConnectionCommand {
            connection_id: connection_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes connection from database
        let connection = WebsocketItem::from_db(&connection_id, &db).await;
        assert!(connection.is_err());

        // Updates session database
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert_eq!(session.modified_action, SessionAction::PendingTimeout);
        assert!(session.modified_at > start_time);

        // Publishes event
        let event_publisher = event_publisher::get().await;
        let messages = event_publisher.get_messages("RustLambda-Dev.Websocket");
        assert_eq!(messages.len(), 1);

        Ok(())
    }
}
