use domain::commands::CreateConnectionCommand;
use domain::errors::LogicError;
use storage::{self, websocket_table::WebsocketItem, IDynamoDbClient};

pub async fn handler(command: &CreateConnectionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let connection = WebsocketItem::new(&command.connection_id);
    db.write_single(connection.save()?).await?;
    Ok("Success".to_string())
}
