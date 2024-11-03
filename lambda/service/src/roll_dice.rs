use crate::{finish_round, send_game_state_notification};
use chrono::Utc;
use domain::commands::{RollDiceCommand, SendGameStateNotificationCommand};
use domain::default_hash_map::DefaultHashMap;
use domain::errors::LogicError;
use rand::Rng;
use std::collections::HashMap;
use std::vec;
use storage::game_table::{
    DiceItem, DiceType, GameAction, GameItem, RollItem, RollResultItem, RollResultNote,
    RollResultType,
};
use storage::session_table::SessionItem;
use storage::IDynamoDbClient;

pub async fn handler(command: &RollDiceCommand) -> Result<String, LogicError> {
    let db = storage::get().await;

    let session = SessionItem::from_db(&command.session_id, &db).await?;
    let game_id = if let Some(game_id) = session.game_id {
        game_id
    } else {
        println!("No game");
        return Ok("No game".to_string());
    };

    let mut game = GameItem::from_db(&game_id, &db).await?;
    let player = game
        .players
        .iter_mut()
        .find(|p| p.player_id == command.session_id)
        .ok_or(LogicError::InvalidGameState(
            "Player not in game".to_string(),
        ))?;
    if player.finished {
        println!("Player already finished");
        return Ok("Player already finished".to_string());
    }
    let is_mr_eleven = if let Some(ref id) = game.mr_eleven {
        id == &player.player_id
    } else {
        false
    };

    let roll = roll_dice(&player.rolls, player.win_counter, &player.nickname);
    player.rolls.push(roll);

    let result = calculate_individual_result(&player.rolls, is_mr_eleven);
    player.finished = result.turn_finished;
    player.outcome = result.note;
    game.version += 1;
    game.modified_action = GameAction::RollDice;
    game.modified_by = command.session_id.clone();
    game.modified_at = Utc::now();

    let is_round_finished = game.players.iter().all(|p| p.finished);
    if is_round_finished {
        finish_round::handler(&mut game)?;
    }

    db.write_single(game.save()?).await?;

    println!("Sending game state notification");
    let command = SendGameStateNotificationCommand {
        game_id: game_id.clone(),
    };
    send_game_state_notification::handler(&command).await?;

    Ok("Ok".to_string())
}

// TODO move these somewhere else, same with storage items
struct RollValues {
    roll_values: Vec<DefaultHashMap<i32, i32>>,
    all_roll_values: DefaultHashMap<i32, i32>,
    sum: i32,
}

pub fn roll_dice(prev_rolls: &[RollItem], win_counter: i32, name: &str) -> RollItem {
    let is_first_roll = prev_rolls.len() == 0;
    let should_roll_death_dice = win_counter >= 3 && is_first_roll;
    let mut dice = vec![];

    dice.push(DiceItem {
        dice_type: DiceType::D6,
        value: random_dice_value(&DiceType::D6),
        is_death_dice: false,
    });
    if is_first_roll {
        dice.push(DiceItem {
            dice_type: DiceType::D6,
            value: random_dice_value(&DiceType::D6),
            is_death_dice: false,
        });
    }
    if should_roll_death_dice {
        let dice_type = get_death_dice_type(&win_counter);
        dice.push(DiceItem {
            dice_type: dice_type,
            value: random_dice_value(&dice_type),
            is_death_dice: true,
        });
    }
    adjust_roll_if_special_name(name, &mut dice);
    RollItem { dice: dice }
}

fn get_death_dice_type(win_counter: &i32) -> DiceType {
    match win_counter {
        3 | 4 => DiceType::D4,
        5 | 6 => DiceType::D6,
        7 | 8 => DiceType::D8,
        9 | 10 => DiceType::D10,
        11 | 12 => DiceType::D12,
        13 | 14 => DiceType::D20,
        _ => DiceType::D10Percentile,
    }
}

fn random_dice_value(dice_type: &DiceType) -> i32 {
    let mut rng = rand::thread_rng();
    match dice_type {
        DiceType::D4 => rng.gen_range(1..=5),
        DiceType::D6 => rng.gen_range(1..=7),
        DiceType::D8 => rng.gen_range(1..=9),
        DiceType::D10 => rng.gen_range(0..=10),
        DiceType::D12 => rng.gen_range(1..=13),
        DiceType::D20 => rng.gen_range(1..=21),
        DiceType::D10Percentile => rng.gen_range(0..=10) * 10,
    }
}

pub fn calculate_individual_result(rolls: &[RollItem], is_mr_eleven: bool) -> RollResultItem {
    let roll_values = count_roll_values(rolls);
    let mut result = RollResultItem::new(
        RollResultNote::None,
        RollResultType::None,
        is_turn_finished(&roll_values),
    );

    // Instant turn finished
    if is_snake_eyes_fail(&roll_values) {
        result.note = RollResultNote::FinishDrink;
        result.result_type = RollResultType::Loser;
        result.turn_finished = true;
    } else if is_snake_eyes_safe(&roll_values) {
        result.note = RollResultNote::SipDrink;
        result.result_type = RollResultType::Loser;
        result.turn_finished = true;
    } else if is_roll_dual_wield(&roll_values) {
        result.note = RollResultNote::DualWield;
        result.turn_finished = true;
    } else if is_roll_shower(&roll_values) {
        result.note = RollResultNote::Shower;
        result.result_type = RollResultType::Loser;
        result.turn_finished = true;
    } else if is_roll_head_on_table(&roll_values) {
        result.note = RollResultNote::HeadOnTable;
        result.turn_finished = true;
    } else if is_roll_wish_purchase(&roll_values) {
        result.note = RollResultNote::WishPurchase;
        result.turn_finished = true;
    } else if is_roll_pool(&roll_values) {
        result.note = RollResultNote::Pool;
        result.result_type = RollResultType::Loser;
        result.turn_finished = true;
    }
    if result.turn_finished && is_mr_eleven && roll_values.sum == 11 {
        if result.note == RollResultNote::None {
            result.note = RollResultNote::Winner;
        }
        result.result_type = RollResultType::Winner;
    }

    if !result.turn_finished {
        let is_uh_oh = is_almost_snake_eyes(&roll_values)
            || is_almost_dual_wield(&roll_values)
            || is_almost_shower(&roll_values)
            || is_almost_head_on_table(&roll_values)
            || is_almost_wish_purchase(&roll_values)
            || is_almost_pool(&roll_values);
        if is_uh_oh {
            result.note = RollResultNote::UhOh;
        }
    }
    result
}

fn is_turn_finished(values: &RollValues) -> bool {
    if values.roll_values.is_empty() {
        return false;
    }
    // If it's the first roll, duplicates grant another roll
    if values.roll_values.len() == 1 {
        for count in values.roll_values[0].values() {
            if count > &1 {
                return false;
            }
        }
        return true;
    }
    // Duplicates with the previous roll grant another roll
    let this_roll = &values.roll_values[values.roll_values.len() - 1];
    let prev_roll = &values.roll_values[values.roll_values.len() - 2];
    for (key, count) in this_roll.iter() {
        if count > &0 && prev_roll.get(key) > &0 {
            return false;
        }
    }
    return true;
}

fn count_roll_values(rolls: &[RollItem]) -> RollValues {
    let mut roll_values = vec![];
    for roll in rolls {
        let mut values = DefaultHashMap::new(0);
        for dice in &roll.dice {
            let count = values.entry(dice.value).or_insert(0);
            *count += 1;
        }
        roll_values.push(values);
    }
    let mut all_roll_values = DefaultHashMap::new(0);
    let mut sum = 0;
    for roll in rolls {
        for dice in &roll.dice {
            let count = all_roll_values.entry(dice.value).or_insert(0);
            *count += 1;
            sum += dice.value;
        }
    }
    RollValues {
        roll_values,
        all_roll_values,
        sum,
    }
}

fn is_almost_snake_eyes(values: &RollValues) -> bool {
    if values.roll_values.len() != 1 {
        return false;
    }
    // First roll must contain two 1's
    values.roll_values[0].get(&1) == &2
}

fn is_snake_eyes_fail(values: &RollValues) -> bool {
    if values.roll_values.is_empty() {
        return false;
    }
    // First roll must contain two 1's
    if values.roll_values[0].get(&1) < &2 {
        return false;
    }
    // If first roll has death dice, three 1's is instant fail
    if values.roll_values[0].get(&1) == &3 {
        return true;
    }
    // The second roll must be 1,2 or 3
    if values.roll_values.len() < 2 {
        return false;
    }
    return values.roll_values[1].get(&1) > &0
        || values.roll_values[1].get(&2) > &0
        || values.roll_values[1].get(&3) > &0;
}

fn is_snake_eyes_safe(values: &RollValues) -> bool {
    if values.roll_values.is_empty() {
        return false;
    }
    // First roll must contain two 1's
    if values.roll_values[0].get(&1) < &2 {
        return false;
    }
    // If first roll has death dice, three 1's is instant fail
    if values.roll_values[0].get(&1) == &3 {
        return false;
    }
    // The second roll must be 4, 5, 6
    if values.roll_values.len() < 2 {
        return false;
    }
    return values.roll_values[1].get(&4) > &0
        || values.roll_values[1].get(&5) > &0
        || values.roll_values[1].get(&6) > &0;
}

fn is_almost_dual_wield(values: &RollValues) -> bool {
    values.all_roll_values.get(&2) == &3
}

fn is_roll_dual_wield(values: &RollValues) -> bool {
    values.all_roll_values.get(&2) == &4
}

fn is_almost_shower(values: &RollValues) -> bool {
    values.all_roll_values.get(&3) == &2
}

fn is_roll_shower(values: &RollValues) -> bool {
    values.all_roll_values.get(&3) == &3
}

fn is_almost_head_on_table(values: &RollValues) -> bool {
    values.all_roll_values.get(&4) == &3
}

fn is_roll_head_on_table(values: &RollValues) -> bool {
    values.all_roll_values.get(&4) == &4
}

fn is_almost_wish_purchase(values: &RollValues) -> bool {
    values.all_roll_values.get(&5) == &4
}

fn is_roll_wish_purchase(values: &RollValues) -> bool {
    values.all_roll_values.get(&5) == &5
}

fn is_almost_pool(values: &RollValues) -> bool {
    values.all_roll_values.get(&6) == &5
}

fn is_roll_pool(values: &RollValues) -> bool {
    values.all_roll_values.get(&6) == &6
}

fn adjust_roll_if_special_name(name: &str, dice: &mut [DiceItem]) {
    let special_names: HashMap<&str, Vec<i32>> = HashMap::from([
        ("SNAKE_EYES", vec![1, 1, 1]),
        ("SNAKE_EYES_SAFE", vec![1, 1, 6]),
        ("DUAL", vec![2, 2, 2, 2]),
        ("DUAL_SPECIAL", vec![2, 2, 3, 2, 2]),
        ("SHOWER", vec![3, 3, 3]),
        ("HEAD", vec![4, 4, 4, 4, 4]),
        ("WISH", vec![5, 5, 5, 5, 5]),
        ("POOL", vec![6, 6, 6, 6, 6, 6]),
        ("MR_ELEVEN", vec![6, 5]),
        ("AVERAGE_JOE", vec![1, 2, 1]),
        ("AVERAGE_PETE", vec![1, 2, 2]),
        ("AVERAGE_GREG", vec![1, 2, 3]),
        ("ABOVE_AVERAGE_JOE", vec![5, 4, 4, 5]),
        ("LUCKY_JOE", vec![6, 6, 5]),
        ("QUANTAM_COCKRING1", vec![5, 3]),
        ("QUANTAM_COCKRING2", vec![3, 5]),
    ]);
    if let Some(rolls) = special_names.get(name) {
        for (d, &r) in dice.iter_mut().zip(rolls.iter()) {
            d.value = r;
        }
    }
}
