use chrono::Utc;
use domain::commands::CreateSessionCommand;
use domain::errors::LogicError;
use notifier::{self, ActionType, INotifier, Message};
use storage::session_table::SessionItem;
use storage::websocket_table::WebsocketItem;
use storage::IDynamoDbClient;
use uuid::Uuid;

pub async fn handler(command: &CreateSessionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let notifier = notifier::get().await;

    let mut connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    let session_id = match connection.session_id {
        Some(session_id) => session_id,
        None => {
            let session_id = Uuid::new_v4().to_string();
            let session = SessionItem::new(&session_id, &command.connection_id);
            connection.session_id = Some(session_id.clone());
            connection.version += 1;
            connection.modified_at = Utc::now();
            db.write(vec![session.save()?, connection.save()?]).await?;
            session_id
        }
    };

    let message = Message::new(ActionType::GetSession(session_id.clone()));
    notifier.notify(&connection.connection_id, &message).await?;
    Ok(session_id)
}
