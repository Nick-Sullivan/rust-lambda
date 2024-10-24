use chrono::Utc;
use domain::commands::DestroyConnectionCommand;
use domain::errors::LogicError;
use event_publisher::{self, EventMessage, IEventPublisher};
use serde_json::json;
use storage::session_table::{SessionAction, SessionItem};
use storage::websocket_table::WebsocketItem;
use storage::IDynamoDbClient;

pub async fn handler(command: &DestroyConnectionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let event_publisher = event_publisher::get().await;

    let connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    match &connection.session_id {
        Some(session_id) => {
            let mut session = SessionItem::from_db(&session_id, &db).await?;
            session.modified_action = SessionAction::PendingTimeout;
            session.modified_at = Utc::now();
            session.version += 1;
            db.write(vec![connection.delete()?, session.save()?])
                .await?;

            let event_message = EventMessage {
                source: "RustLambda-Dev.Websocket".to_string(),
                detail_type: "Disconnected".to_string(),
                detail: json!({"session_id": session_id}),
            };
            event_publisher.publish(&event_message).await?;
        }
        None => {
            db.write_single(connection.delete()?).await?;
        }
    }
    Ok("Success".to_string())
}
