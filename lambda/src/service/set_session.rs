use crate::dependency_injection::get_session_table;
use crate::domain::commands::SetSessionCommand;
use crate::domain::errors::LogicError;
use crate::storage::session_table::{ISessionTable, SessionItem};

pub async fn handler(command: &SetSessionCommand) -> Result<String, LogicError> {
    println!("getting table");

    let db = get_session_table().await;
    println!("locking");
    let mut db_lock = db.lock().await;
    println!("initialising item");
    let item = SessionItem::new(&command.session_id, &command.connection_id);
    println!("saving to db");
    db_lock.save(&item).await?;
    println!("returning");
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create() {
        let session_id = "session_id".to_string();
        let connection_id = "connection_id".to_string();
        let request = SetSessionCommand {
            session_id,
            connection_id,
        };
        let result = handler(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update() {
        let session_id = "session_id".to_string();
        let connection_id = "connection_id".to_string();
        let item = SessionItem::new(&session_id, &connection_id);
        let db = get_session_table().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.save(&item).await;
        drop(db_lock);

        let request = SetSessionCommand {
            session_id,
            connection_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }
}
