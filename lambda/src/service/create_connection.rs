use crate::dependency_injection::get_websocket_table;
use crate::domain::commands::CreateConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::websocket_table::{IWebsocketTable, WebsocketItem};

pub async fn handler(command: &CreateConnectionCommand) -> Result<String, LogicError> {
    println!("Creating connection");
    let db = get_websocket_table().await;
    let mut db_lock = db.lock().await;
    let item = WebsocketItem::new(&command.connection_id);
    db_lock.save(&item).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create() {
        let connection_id = "id".to_string();
        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_already_exists() {
        let connection_id = "id".to_string();
        let item = WebsocketItem::new(&connection_id);
        let db = get_websocket_table().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.save(&item).await;
        drop(db_lock);

        let request = CreateConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_err());
    }
}
