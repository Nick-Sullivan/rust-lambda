use domain::commands::SendGameStateNotificationCommand;
use domain::errors::LogicError;
use notifier::{self, ActionType, GameStateMessage, INotifier, Message, RoundStateMessage};
use storage::{game_table::GameItem, session_table::SessionItem};

pub async fn handler(command: &SendGameStateNotificationCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let notifier = notifier::get().await;

    let game = GameItem::from_db(&command.game_id, &db).await?;

    let game_message = GameStateMessage {
        game_id: game.game_id.clone(),
        round: RoundStateMessage {
            complete: game.round_finished,
        },
    };
    let message = Message::new(ActionType::GameState(game_message));

    let session_ids = game
        .players
        .iter()
        .map(|p| p.player_id.clone())
        .collect::<Vec<_>>();

    for session_id in session_ids {
        let session = SessionItem::from_db(&session_id, &db).await?;
        notifier.notify(&session.connection_id, &message).await?;
    }

    Ok(command.game_id.clone())
}
