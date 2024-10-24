use chrono::Utc;
use domain::commands::{DestroySessionCommand, LeaveGameCommand};
use domain::errors::LogicError;
use notifier::{self, ActionType, INotifier, Message};
use storage::session_table::SessionItem;
use storage::IDynamoDbClient;

use crate::leave_game;

pub async fn handler(command: &DestroySessionCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let notifier = notifier::get().await;

    let mut session = SessionItem::from_db(&command.session_id, &db).await?;
    session.version += 1;
    session.modified_at = Utc::now();

    if let Some(game_id) = &session.game_id {
        let request = LeaveGameCommand {
            game_id: game_id.clone(),
            session_id: session.session_id.clone(),
        };
        leave_game::handler(&request).await?;
    }

    db.write_single(session.delete()?).await?;

    if let Some(connection_id) = &command.connection_id {
        let message = Message::new(ActionType::DestroySession(command.session_id.clone()));
        notifier.notify(&connection_id, &message).await?;
    }
    Ok("Success".to_string())
}
