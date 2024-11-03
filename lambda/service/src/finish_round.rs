use domain::errors::LogicError;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use storage::game_table::{GameItem, RollResultNote, RollResultType};

pub fn handler(game: &mut GameItem) -> Result<(), LogicError> {
    for player in &game.players {
        if !player.finished {
            return Ok(());
        }
    }
    let result = calculate_round_results(game);
    game.mr_eleven = calculate_new_mr_eleven(&result);
    game.round_finished = result.finished;
    for player in &mut game.players {
        let player_result = &result.player_scores[&player.player_id];
        player.outcome = player_result.outcome;
        player.outcome_type = player_result.outcome_type;
        player.finished = player_result.finished;
        player.win_counter = match player_result.outcome_type {
            RollResultType::Winner => player.win_counter + 1,
            RollResultType::NoChange => player.win_counter,
            RollResultType::Loser => 0,
            RollResultType::None => {
                return Err(LogicError::InvalidGameState(
                    "Unexpected outcome type".to_string(),
                ))
            }
        }
    }
    Ok(())
}

struct RoundResult {
    players_in_contention: Vec<String>,
    player_scores: HashMap<String, PlayerScore>,
    finished: bool,
    mr_eleven: Option<String>,
}
struct PlayerScore {
    outcome: RollResultNote,
    outcome_type: RollResultType,
    score: i32,
    finished: bool,
}

fn calculate_round_results(game: &GameItem) -> RoundResult {
    let mut result = create_round_result(&game);
    handle_instant_loss(&mut result);
    handle_mr_eleven(&mut result);
    let max_value = if let Some(max_value) = get_contenders_max_value(&result) {
        max_value
    } else {
        return result;
    };
    let player_ids = get_players_with_value(&result, max_value);
    match player_ids.len() {
        1 => handle_highest_value(&mut result, max_value),
        3 => handle_three_way_tie(&mut result, max_value),
        _ => handle_tie(&mut result, max_value),
    }
    result
}

fn create_round_result(game: &GameItem) -> RoundResult {
    let mut player_scores: HashMap<String, PlayerScore> = HashMap::new();
    for player in &game.players {
        let score = player
            .rolls
            .iter()
            .flat_map(|roll| roll.dice.iter())
            .map(|dice| dice.value)
            .sum();
        player_scores.insert(
            player.player_id.clone(),
            PlayerScore {
                score: score,
                outcome: player.outcome,
                outcome_type: player.outcome_type,
                finished: player.finished,
            },
        );
    }
    RoundResult {
        players_in_contention: player_scores.keys().cloned().collect(),
        player_scores,
        finished: true,
        mr_eleven: game.mr_eleven.clone(),
    }
}

fn get_contenders_max_value(result: &RoundResult) -> Option<i32> {
    let values: Vec<i32> = result
        .players_in_contention
        .iter()
        .map(|player_id| result.player_scores[player_id].score)
        .collect();
    values.iter().max().cloned()
}

fn get_players_with_value(result: &RoundResult, value: i32) -> Vec<String> {
    result
        .player_scores
        .iter()
        .filter(|(_, score)| score.score == value)
        .map(|(player_id, _)| player_id.clone())
        .collect()
}

fn handle_instant_loss(result: &mut RoundResult) {
    if result.players_in_contention.is_empty() {
        return;
    }
    println!("Removing instant loss players");
    let losers: Vec<String> = result
        .player_scores
        .iter()
        .filter(|(_, score)| score.outcome_type == RollResultType::Loser)
        .map(|(player_id, _)| player_id.clone())
        .collect();
    result
        .players_in_contention
        .retain(|player_id| !losers.contains(player_id));
}

fn handle_mr_eleven(result: &mut RoundResult) {
    if result.players_in_contention.is_empty() {
        return;
    }
    let mr_eleven = if let Some(mr_eleven) = &result.mr_eleven {
        mr_eleven
    } else {
        return;
    };
    if result.player_scores[mr_eleven].score != 11 {
        return;
    }
    println!("Mr Eleven wins");
    for player_id in &result.players_in_contention {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player_id == mr_eleven {
            player.outcome = RollResultNote::Winner;
            player.outcome_type = RollResultType::Winner;
        } else {
            if player.outcome == RollResultNote::None {
                player.outcome = RollResultNote::SipDrink;
            }
            player.outcome_type = RollResultType::Loser;
        }
    }
    result.players_in_contention = vec![];
}

fn handle_three_way_tie(result: &mut RoundResult, max_value: i32) {
    let (tied_players, lose_players): (Vec<&String>, Vec<&String>) = result
        .players_in_contention
        .iter()
        .partition(|&player_id| result.player_scores[player_id].score == max_value);
    for player_id in tied_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = RollResultNote::ThreeWayTie;
        }
        player.outcome_type = RollResultType::NoChange;
        player.finished = false;
    }
    for player_id in lose_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = RollResultNote::SipDrink;
        }
        player.outcome_type = RollResultType::Loser;
    }
    result.finished = false;
    result.players_in_contention = vec![];
}

fn handle_tie(result: &mut RoundResult, max_value: i32) {
    let (tied_players, lose_players): (Vec<&String>, Vec<&String>) = result
        .players_in_contention
        .iter()
        .partition(|&player_id| result.player_scores[player_id].score == max_value);
    let note = if max_value == 8 {
        RollResultNote::CockringHands
    } else {
        RollResultNote::Tie
    };
    for player_id in tied_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = note;
        }
        player.outcome_type = RollResultType::Loser;
    }
    for player_id in lose_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = RollResultNote::SipDrink;
        }
        player.outcome_type = RollResultType::Loser;
    }
    result.players_in_contention = vec![];
}

fn handle_highest_value(result: &mut RoundResult, max_value: i32) {
    let (win_players, lose_players): (Vec<&String>, Vec<&String>) = result
        .players_in_contention
        .iter()
        .partition(|&player_id| result.player_scores[player_id].score == max_value);
    for player_id in win_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = RollResultNote::Winner;
        }
        player.outcome_type = RollResultType::Winner;
    }
    for player_id in lose_players {
        let player = result.player_scores.get_mut(player_id).unwrap();
        if player.outcome == RollResultNote::None {
            player.outcome = RollResultNote::SipDrink;
        }
        player.outcome_type = RollResultType::Loser;
    }
    result.players_in_contention = vec![];
}

fn calculate_new_mr_eleven(result: &RoundResult) -> Option<String> {
    let players_with_eleven = result
        .player_scores
        .iter()
        .filter(|(_, score)| score.score == 11)
        .map(|(player_id, _)| player_id.clone())
        .collect::<Vec<String>>();

    if let Some(mr_eleven) = &result.mr_eleven {
        if players_with_eleven.contains(&mr_eleven) {
            return Some(mr_eleven.clone());
        }
        if players_with_eleven.is_empty() {
            return Some(mr_eleven.clone());
        }
    }
    if players_with_eleven.is_empty() {
        return None;
    }
    players_with_eleven.choose(&mut thread_rng()).cloned()
}
