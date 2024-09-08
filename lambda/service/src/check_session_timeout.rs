use crate::destroy_session;
use chrono::Utc;
use domain::commands::{CheckSessionTimeoutCommand, DestroySessionCommand};
use domain::errors::LogicError;
use storage::session_table::{SessionAction, SessionItem};

pub async fn handler(command: &CheckSessionTimeoutCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let session = SessionItem::from_db(&command.session_id, &db).await;
    let session = match session {
        Ok(session) => session,
        Err(LogicError::GetItemError(_)) => return Ok("Session already deleted".to_string()),
        Err(e) => return Err(e),
    };
    if session.modified_action != SessionAction::PendingTimeout {
        return Ok("Session is not pending timeout".to_string());
    }
    let now = Utc::now();
    let seconds_since_disconnected = (now - session.modified_at).num_seconds();
    let seconds_timeout = 30;
    if seconds_since_disconnected < seconds_timeout {
        return Ok("Session is not timed out".to_string());
    }

    let command = DestroySessionCommand {
        connection_id: None,
        session_id: command.session_id.clone(),
    };
    destroy_session::handler(&command).await?;

    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_setup;
    use storage::IDynamoDbClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn does_nothing_if_session_disconnected() -> Result<(), LogicError> {
        test_setup::setup();
        let session_id = Uuid::new_v4().to_string();
        let request = CheckSessionTimeoutCommand { session_id };
        let result = handler(&request).await;
        let expected_msg = "Session already deleted";
        match result {
            Ok(message) => assert_eq!(message, expected_msg, "Unexpected message: {}", message),
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_if_session_reconnected() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);
        db.write_single(session.save()?).await?;

        let request = CheckSessionTimeoutCommand {
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;
        let expected_msg = "Session is not pending timeout";
        match result {
            Ok(message) => assert_eq!(message, expected_msg, "Unexpected message: {}", message),
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_if_session_hasnt_had_enough_time_to_reconnect() -> Result<(), LogicError>
    {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.modified_action = SessionAction::PendingTimeout;
        session.modified_at = Utc::now();
        db.write_single(session.save()?).await?;

        let request = CheckSessionTimeoutCommand {
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;
        let expected_msg = "Session is not timed out";
        match result {
            Ok(message) => assert_eq!(message, expected_msg, "Unexpected message: {}", message),
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
        Ok(())
    }

    #[tokio::test]
    async fn destroys_session() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.modified_action = SessionAction::PendingTimeout;
        session.modified_at = Utc::now() - chrono::Duration::seconds(31);
        db.write_single(session.save()?).await?;

        let request = CheckSessionTimeoutCommand {
            session_id: session_id.clone(),
        };
        let result = handler(&request).await;
        assert!(result.is_ok(), "Error: {:?}", result.err());

        // Deletes database item
        let session = SessionItem::from_db(&session_id, &db).await;
        assert!(session.is_err());
        Ok(())
    }
}
