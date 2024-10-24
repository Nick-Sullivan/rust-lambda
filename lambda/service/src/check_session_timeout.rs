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
