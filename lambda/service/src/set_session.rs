use chrono::Utc;
use domain::commands::SetSessionCommand;
use domain::errors::LogicError;
use notifier::{self, ActionType, INotifier, Message};
use storage::session_table::{SessionAction, SessionItem};
use storage::websocket_table::WebsocketItem;
use storage::IDynamoDbClient;

pub async fn handler(command: &SetSessionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let notifier = notifier::get().await;

    // TODO: if session doesnt exist, create a new one instead

    let mut connection = WebsocketItem::from_db(&command.connection_id, &db).await?;
    connection.session_id = Some(command.session_id.clone());
    connection.version += 1;
    connection.modified_at = Utc::now();

    let mut session = SessionItem::from_db(&command.session_id, &db).await?;
    session.connection_id = command.connection_id.clone();
    session.version += 1;
    session.modified_at = Utc::now();
    session.modified_action = SessionAction::Reconnected;

    println!("Saving to database");
    db.write(vec![session.save()?, connection.save()?]).await?;

    println!("Notifying connections");
    let message = Message::new(ActionType::GetSession(command.session_id.clone()));
    notifier.notify(&connection.connection_id, &message).await?;

    println!("Returning");
    Ok(command.session_id.clone())
}
