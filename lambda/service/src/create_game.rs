use crate::send_game_state_notification;
use chrono::Utc;
use domain::commands::{CreateGameCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use event_publisher::{self, EventMessage, IEventPublisher};
use notifier::{self, ActionType, INotifier, Message};
use serde_json::json;
use storage::game_table::{GameItem, PlayerItem};
use storage::session_table::{SessionAction, SessionItem};
use storage::IDynamoDbClient;

pub async fn handler(command: &CreateGameCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let event_publisher = event_publisher::get().await;
    let notifier = notifier::get().await;

    let mut session = SessionItem::from_db(&command.session_id, &db).await?;
    let nickname = session.nickname.clone().ok_or(LogicError::NotAllowed)?;
    if session.game_id.is_some() {
        return Ok("Already in game".to_string());
    }

    let game_id = GameItem::create_game_code();
    let mut game = GameItem::new(&game_id, &command.session_id);
    let player = PlayerItem::new(&session.session_id, &session.account_id, &nickname);
    game.players.push(player);
    session.game_id = Some(game_id.clone());
    session.modified_action = SessionAction::JoinGame;
    session.modified_at = Utc::now();
    session.version += 1;

    db.write(vec![game.save()?, session.save()?]).await?;

    println!("Sending new game response");
    let message = Message::new(ActionType::JoinGame(game_id.clone()));
    notifier.notify(&command.connection_id, &message).await?;

    println!("Sending game state notification");
    let command = SendGameStateNotificationCommand {
        game_id: game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    println!("Sending event message");
    let event_message = EventMessage {
        source: "RustLambda-Dev.GameCreated".to_string(),
        detail_type: "Game created".to_string(),
        detail: json!({"game_id": game_id}),
    };
    event_publisher.publish(&event_message).await?;

    Ok(game_id)
}
