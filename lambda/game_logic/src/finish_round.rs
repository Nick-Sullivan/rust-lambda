use domain::errors::LogicError;
use storage::game_table::GameItem;

pub fn finish_round(game: &mut GameItem) -> Result<GameItem, LogicError> {
    if !game.players.iter().all(|p| p.finished) {
        return Err(LogicError::InvalidGameState(
            "Not all players have finished".to_string(),
        ));
    }
    game.round_finished = true;

    Ok(game.clone())
}
