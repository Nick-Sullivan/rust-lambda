use chrono::Utc;
use domain::commands::{LeaveGameCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use game_logic::finish_round;
use storage::game_table::{GameAction, GameItem};
use storage::IDynamoDbClient;

use crate::send_game_state_notification;

pub async fn handler(command: &LeaveGameCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let mut game = GameItem::from_db(&command.game_id, &db).await?;
    game.players.retain(|p| p.player_id != command.session_id);
    game.version += 1;
    game.modified_at = Utc::now();
    game.modified_action = GameAction::LeaveGame;

    if game.players.is_empty() {
        println!("No more players");
        db.write_single(game.delete()?).await?;
        return Ok("Success".to_string());
    }
    if game.players.iter().all(|p| p.finished) {
        println!("All players finished");
        game = finish_round::finish_round(&mut game)?;
    }

    db.write_single(game.save()?).await?;

    let command = SendGameStateNotificationCommand {
        game_id: game.game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    Ok("Success".to_string())
}
