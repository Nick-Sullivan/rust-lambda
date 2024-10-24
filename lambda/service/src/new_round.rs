use crate::send_game_state_notification;
use chrono::Utc;
use domain::commands::{NewRoundCommand, SendGameStateNotificationCommand};
use domain::errors::LogicError;
use storage::game_table::{GameAction, GameItem};
use storage::session_table::SessionItem;
use storage::IDynamoDbClient;

pub async fn handler(command: &NewRoundCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let session = SessionItem::from_db(&command.session_id, &db).await?;

    let game_id = if let Some(game_id) = session.game_id {
        game_id
    } else {
        println!("No game");
        return Ok("No game".to_string());
    };

    let mut game = GameItem::from_db(&game_id, &db).await?;
    if !game.round_finished {
        println!("Round not finished");
        return Ok("Round not finished".to_string());
    }

    game.round_finished = false;
    game.version += 1;
    // game.round_id += 1;
    game.modified_action = GameAction::NewRound;
    game.modified_by = command.session_id.clone();
    game.modified_at = Utc::now();
    for player in game.players.iter_mut() {
        player.finished = false;
        // player.outcome = ;
        // player.rolls = Vec::new();
    }
    db.write_single(game.save()?).await?;

    println!("Sending game state notification");
    let command = SendGameStateNotificationCommand {
        game_id: game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    Ok(game_id)
}
