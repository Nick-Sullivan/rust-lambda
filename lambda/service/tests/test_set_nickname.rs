mod test_setup;

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use chrono::Utc;
    use domain::commands::SetNicknameCommand;
    use domain::errors::LogicError;
    use notifier::{self, INotifier};
    use service::set_nickname::handler;
    use storage::session_table::{SessionAction, SessionItem};
    use storage::IDynamoDbClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn errors_if_session_doesnt_exist() {
        test_setup::setup();
        let nickname = "nickname".to_string();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let account_id = None;
        let request = SetNicknameCommand {
            account_id,
            connection_id,
            session_id,
            nickname,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn aborts_if_nickname_is_invalid() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);

        db.write_single(session.save()?).await?;

        let nickname = "1".to_string();
        let request = SetNicknameCommand {
            account_id: None,
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
            nickname: nickname.clone(),
        };
        let result = handler(&request).await;

        // Returns OK
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Does not update database tables
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert!(session.nickname.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn updates_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let account_id = Uuid::new_v4().to_string();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);

        db.write_single(session.save()?).await?;

        let nickname = "nickname".to_string();
        let request = SetNicknameCommand {
            account_id: Some(account_id.clone()),
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
            nickname: nickname.clone(),
        };
        let result = handler(&request).await;

        // Returns OK
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Updates database tables
        let session = SessionItem::from_db(&session_id, &db).await?;
        assert!(session.account_id.is_some());
        assert_eq!(session.account_id.unwrap(), account_id);
        assert!(session.nickname.is_some());
        assert_eq!(session.nickname.unwrap(), nickname);
        assert!(session.modified_at > start_time);
        assert_eq!(session.modified_action, SessionAction::SetNickname);
        Ok(())
    }
}
