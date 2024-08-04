use crate::dependency_injection::get_websocket_table;
use crate::domain::commands::DestroyConnectionCommand;
use crate::domain::errors::LogicError;
use crate::storage::websocket_table::IWebsocketTable;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    println!("Destroying connection");
    let db = get_websocket_table().await;
    let mut db_lock = db.lock().await;
    db_lock.clear(&command.connection_id).await?;
    Ok("Success".to_string())
}

#[cfg(test)]
mod tests {
    use crate::storage::websocket_table::WebsocketItem;

    use super::*;

    #[tokio::test]
    async fn test_doesnt_exist() {
        let connection_id = "id".to_string();
        let request = DestroyConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_destroy() {
        let connection_id = "id".to_string();
        let item = WebsocketItem {
            connection_id: connection_id.to_string(),
            version: 0,
        };
        let db = get_websocket_table().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.save(&item).await;
        drop(db_lock);

        let request = DestroyConnectionCommand { connection_id };
        let result = handler(&request).await;
        assert!(result.is_ok());
    }
}
