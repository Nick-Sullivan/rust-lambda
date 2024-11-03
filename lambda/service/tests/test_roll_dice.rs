mod test_setup;

#[cfg(test)]
mod test {
    use crate::test_setup;
    use chrono::Utc;
    use domain::commands::RollDiceCommand;
    use domain::errors::LogicError;
    use notifier::{self, INotifier};
    use rstest::rstest;
    use service::roll_dice::{calculate_individual_result, handler, roll_dice};
    use std::vec;
    use storage::game_table::{DiceItem, DiceType, GameAction, GameItem, PlayerItem, RollItem, RollResultItem, RollResultNote, RollResultType};
    use storage::session_table::SessionItem;
    use storage::IDynamoDbClient;
    use uuid::Uuid;

    const D4: DiceType = DiceType::D4;
    const D6: DiceType = DiceType::D6;

    #[tokio::test]
    async fn errors_if_session_doesnt_exist() {
        test_setup::setup();
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let request = RollDiceCommand {
            connection_id,
            session_id,
        };
        let result = handler(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn does_nothing_if_game_doesnt_exist() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let session = SessionItem::new(&session_id, &connection_id);
        db.write_single(session.save()?).await?;

        let request = RollDiceCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // No notifications
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_if_player_is_finished() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let game_id = Uuid::new_v4().to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.game_id = Some(game_id.clone());
        let mut game = GameItem::new(&game_id, &session_id);
        game.round_finished = false;
        let mut player = PlayerItem::new(&session.session_id, &session.account_id, "Test");
        player.finished = true;
        game.players.push(player);

        db.write(vec![session.save()?, game.save()?]).await?;

        let request = RollDiceCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // No notifications
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn creates_new_roll() -> Result<(), LogicError> {
        test_setup::setup();
        let db = storage::get().await;
        let start_time = Utc::now();

        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let game_id = Uuid::new_v4().to_string();
        let nickname = "AVERAGE_JOE".to_string();
        let mut session = SessionItem::new(&session_id, &connection_id);
        session.game_id = Some(game_id.clone());
        session.nickname = Some(nickname.clone());
        let mut game = GameItem::new(&game_id, &session_id);
        let player = PlayerItem::new(&session.session_id, &session.account_id, &nickname);
        game.players.push(player);
        game.round_finished = false;
        db.write(vec![session.save()?, game.save()?]).await?;

        let request = RollDiceCommand {
            connection_id: connection_id.clone(),
            session_id: session_id.clone(),
        };
        handler(&request).await?;

        // Notifies the connection
        let notifier = notifier::get().await;
        let messages = notifier.get_messages(&connection_id);
        assert_eq!(messages.len(), 1);

        // Updates game item
        let game = GameItem::from_db(&game_id, &db).await?;
        assert!(game.modified_at > start_time);
        assert_eq!(game.modified_action, GameAction::RollDice);
        assert_eq!(game.modified_by, session_id);
        assert_eq!(game.round_finished, true);
        assert_eq!(game.version, 1);
        let rolls = &game.players[0].rolls;
        assert_eq!(rolls.len(), 1);
        let dice = &rolls[0].dice;
        assert_eq!(dice.len(), 2);
        assert_eq!(dice[0].value, 1);
        assert_eq!(dice[1].value, 2);

        Ok(())
    }

    #[rstest]
    #[case::first_roll(0, vec![], vec![D6, D6])]
    #[case::first_roll_with_death_dice(4, vec![], vec![D6, D6, D4])]
    #[case::second_roll(
        0, 
        vec![RollItem{dice:vec![
            DiceItem{dice_type: D6, value: 1, is_death_dice: false},
            DiceItem{dice_type: D6, value: 2, is_death_dice: false},
        ]}],
        vec![D6]
    )]
    #[case::second_roll_with_death_dice(
        5, 
        vec![RollItem{dice:vec![
            DiceItem{dice_type: D6, value: 1, is_death_dice: false},
            DiceItem{dice_type: D6, value: 2, is_death_dice: false},
            DiceItem{dice_type: D6, value: 2, is_death_dice: true},
        ]}],
        vec![D6]
    )]
    fn uses_correct_dice_size(
        #[case] wins: i32,
        #[case] prev_rolls: Vec<RollItem>,
        #[case] expected: Vec<DiceType>,
    ) {
        let name = "name";
        let roll = roll_dice(&prev_rolls, wins, name);
        assert_eq!(roll.dice.len(), expected.len());
        for i in 0..expected.len() {
            assert_eq!(roll.dice[i].dice_type, expected[i]);
        }
    }

    #[rstest]
    #[case(
        "ABOVE_AVERAGE_JOE", 
        vec![
            DiceItem{dice_type: D6, value: 5, is_death_dice: false},
            DiceItem{dice_type: D6, value: 4, is_death_dice: false},
        ],
    )]
    fn uses_special_names(#[case] name: &str, #[case] expected: Vec<DiceItem>) {
        let roll = roll_dice(&vec![], 0, name);
        assert_eq!(roll.dice.len(), expected.len());
        for i in 0..expected.len() {
            assert_eq!(roll.dice[i], expected[i]);
        }
    }

    #[rstest]
    #[case::snake_eyes(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 1), DiceItem::new(D6, 1)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::snake_eyes_fail(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 1), DiceItem::new(D6, 1)]},
            RollItem{dice: vec![DiceItem::new(D6, 1)]}
        ],
        RollResultItem::new(RollResultNote::FinishDrink, RollResultType::Loser, true),
    )]
    #[case::snake_eyes_pass(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 1), DiceItem::new(D6, 1)]},
            RollItem{dice: vec![DiceItem::new(D6, 4)]}
        ],
        RollResultItem::new(RollResultNote::SipDrink, RollResultType::Loser, true),
    )]
    #[case::dual_wield_warn(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 2), DiceItem::new(D6, 2)]},
            RollItem{dice: vec![DiceItem::new(D6, 2)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::dual_wield(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 2), DiceItem::new(D6, 2)]},
            RollItem{dice: vec![DiceItem::new(D6, 2)]},
            RollItem{dice: vec![DiceItem::new(D6, 2)]},
        ],
        RollResultItem::new(RollResultNote::DualWield, RollResultType::None, true),
    )]
    #[case::shower_warn(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 3), DiceItem::new(D6, 3)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::shower(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 3), DiceItem::new(D6, 3)]},
            RollItem{dice: vec![DiceItem::new(D6, 3)]},
        ],
        RollResultItem::new(RollResultNote::Shower, RollResultType::Loser, true),
    )]
    #[case::head_on_table_warn(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 4), DiceItem::new(D6, 4)]},
            RollItem{dice: vec![DiceItem::new(D6, 4)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::head_on_table(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 4), DiceItem::new(D6, 4)]},
            RollItem{dice: vec![DiceItem::new(D6, 4)]},
            RollItem{dice: vec![DiceItem::new(D6, 4)]},
        ],
        RollResultItem::new(RollResultNote::HeadOnTable, RollResultType::None, true),
    )]
    #[case::wish_warn(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 5), DiceItem::new(D6, 5)]},
            RollItem{dice: vec![DiceItem::new(D6, 5)]},
            RollItem{dice: vec![DiceItem::new(D6, 5)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::wish(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 5), DiceItem::new(D6, 5)]},
            RollItem{dice: vec![DiceItem::new(D6, 5)]},
            RollItem{dice: vec![DiceItem::new(D6, 5)]},
            RollItem{dice: vec![DiceItem::new(D6, 5)]},
        ],
        RollResultItem::new(RollResultNote::WishPurchase, RollResultType::None, true),
    )]
    #[case::pool_warn(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 6), DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
        ],
        RollResultItem::new(RollResultNote::UhOh, RollResultType::None, false),
    )]
    #[case::pool(
        vec![
            RollItem{dice: vec![DiceItem::new(D6, 6), DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
            RollItem{dice: vec![DiceItem::new(D6, 6)]},
        ],
        RollResultItem::new(RollResultNote::Pool, RollResultType::Loser, true),
    )]
    #[case::instant_finish_drink(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 1),
                DiceItem::new(D6, 1),
                DiceItem{dice_type: D4, value: 1, is_death_dice: true},
            ]},
        ],
        RollResultItem::new(RollResultNote::FinishDrink, RollResultType::Loser, true),
    )]
    #[case::regular_roll(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 4),
                DiceItem::new(D6, 5),
                DiceItem{dice_type: D4, value: 2, is_death_dice: true},
            ]},
        ],
        RollResultItem::new(RollResultNote::None, RollResultType::None, true),
    )]
    fn calculates_individual_result(#[case] rolls: Vec<RollItem>, #[case] expected: RollResultItem) {
        let result = calculate_individual_result(&rolls, false);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::rolls_eleven(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 4),
                DiceItem::new(D6, 5),
                DiceItem{dice_type: D4, value: 2, is_death_dice: true},
            ]},
        ],
        RollResultItem::new(RollResultNote::Winner, RollResultType::Winner, true),
    )]
    #[case::snake_eyes_and_eleven(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 1),
                DiceItem::new(D6, 1),
                DiceItem{dice_type: D6, value: 6, is_death_dice: true},
            ]},
            RollItem{dice: vec![DiceItem::new(D6, 3)]},
        ],
        RollResultItem::new(RollResultNote::FinishDrink, RollResultType::Winner, true),
    )]
    #[case::dual_wield_and_eleven(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 2),
                DiceItem::new(D6, 2),
                DiceItem{dice_type: D4, value: 3, is_death_dice: true},
            ]},
            RollItem{dice: vec![DiceItem::new(D6, 2)]},
            RollItem{dice: vec![DiceItem::new(D6, 2)]},
        ],
        RollResultItem::new(RollResultNote::DualWield, RollResultType::Winner, true),
    )]
    #[case::shower_and_eleven(
        vec![
            RollItem{dice: vec![
                DiceItem::new(D6, 3),
                DiceItem::new(D6, 3),
                DiceItem{dice_type: D4, value: 2, is_death_dice: true},
            ]},
            RollItem{dice: vec![DiceItem::new(D6, 3)]},
        ],
        RollResultItem::new(RollResultNote::Shower, RollResultType::Winner, true),
    )]
    fn calculates_individual_result_as_mr_eleven(#[case] rolls: Vec<RollItem>, #[case] expected: RollResultItem) {
        let result = calculate_individual_result(&rolls, true);
        assert_eq!(result, expected);
    }
}
