#[cfg(test)]
mod test {
    use domain::errors::LogicError;
    use service::finish_round::handler;
    use std::vec;
    use storage::game_table::{
        DiceItem, DiceType, GameItem, PlayerItem, RollItem, RollResultNote, RollResultType,
    };

    #[test]
    fn does_nothing_if_players_arent_finished() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.players = vec![PlayerItem {
            player_id: "player_id".to_string(),
            account_id: None,
            nickname: "nickname".to_string(),
            win_counter: 0,
            rolls: vec![],
            outcome: RollResultNote::None,
            outcome_type: RollResultType::None,
            finished: false,
        }];
        handler(&mut game)?;
        // No update
        assert_eq!(game.round_finished, false);
        Ok(())
    }

    #[test]
    fn finishes_highest_value() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.players = vec![
            PlayerItem {
                player_id: "player_1".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 6),
                        DiceItem::new(DiceType::D6, 5),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_2".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 3),
                        DiceItem::new(DiceType::D6, 2),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
        ];
        handler(&mut game)?;
        assert_eq!(game.round_finished, true);
        assert_eq!(game.mr_eleven, Some("player_1".to_string()));
        assert_eq!(game.players[0].outcome, RollResultNote::Winner);
        assert_eq!(game.players[0].outcome_type, RollResultType::Winner);
        assert_eq!(game.players[0].win_counter, 1);
        assert_eq!(game.players[1].outcome, RollResultNote::SipDrink);
        assert_eq!(game.players[1].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[1].win_counter, 0);
        Ok(())
    }

    #[test]
    fn finishes_tie() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.players = vec![
            PlayerItem {
                player_id: "player_1".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 3),
                        DiceItem::new(DiceType::D6, 2),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_2".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 3),
                        DiceItem::new(DiceType::D6, 2),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_3".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![
                    RollItem {
                        dice: vec![
                            DiceItem::new(DiceType::D6, 3),
                            DiceItem::new(DiceType::D6, 3),
                        ],
                    },
                    RollItem {
                        dice: vec![DiceItem::new(DiceType::D6, 3)],
                    },
                ],
                outcome: RollResultNote::Shower,
                outcome_type: RollResultType::Loser,
                finished: true,
            },
        ];
        handler(&mut game)?;
        assert_eq!(game.round_finished, true);
        assert_eq!(game.mr_eleven, None);
        assert_eq!(game.players[0].outcome, RollResultNote::Tie);
        assert_eq!(game.players[0].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[0].win_counter, 0);
        assert_eq!(game.players[1].outcome, RollResultNote::Tie);
        assert_eq!(game.players[1].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[1].win_counter, 0);
        assert_eq!(game.players[2].outcome, RollResultNote::Shower);
        assert_eq!(game.players[2].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[2].win_counter, 0);
        Ok(())
    }

    #[test]
    fn finishes_three_way_tie() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.players = vec![
            PlayerItem {
                player_id: "player_1".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 2),
                        DiceItem::new(DiceType::D6, 5),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_2".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 2,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 4),
                        DiceItem::new(DiceType::D6, 3),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_3".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 1),
                        DiceItem::new(DiceType::D6, 6),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_4".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 1),
                        DiceItem::new(DiceType::D6, 2),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
        ];
        handler(&mut game)?;
        assert_eq!(game.round_finished, false);
        assert_eq!(game.mr_eleven, None);
        assert_eq!(game.players[0].outcome, RollResultNote::ThreeWayTie);
        assert_eq!(game.players[0].outcome_type, RollResultType::NoChange);
        assert_eq!(game.players[0].win_counter, 0);
        assert_eq!(game.players[0].finished, false);
        assert_eq!(game.players[1].outcome, RollResultNote::ThreeWayTie);
        assert_eq!(game.players[1].outcome_type, RollResultType::NoChange);
        assert_eq!(game.players[1].win_counter, 2);
        assert_eq!(game.players[1].finished, false);
        assert_eq!(game.players[2].outcome, RollResultNote::ThreeWayTie);
        assert_eq!(game.players[2].outcome_type, RollResultType::NoChange);
        assert_eq!(game.players[2].finished, false);
        assert_eq!(game.players[2].win_counter, 0);
        assert_eq!(game.players[3].outcome, RollResultNote::SipDrink);
        assert_eq!(game.players[3].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[3].win_counter, 0);
        assert_eq!(game.players[3].finished, true);
        Ok(())
    }

    #[test]
    fn finishes_mr_eleven() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.mr_eleven = Some("player_1".to_string());
        game.players = vec![
            PlayerItem {
                player_id: "player_1".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 5),
                        DiceItem::new(DiceType::D6, 6),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
            PlayerItem {
                player_id: "player_2".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 3,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 6),
                        DiceItem::new(DiceType::D6, 5),
                        DiceItem::new(DiceType::D4, 2),
                    ],
                }],
                outcome: RollResultNote::None,
                outcome_type: RollResultType::None,
                finished: true,
            },
        ];
        handler(&mut game)?;
        assert_eq!(game.round_finished, true);
        assert_eq!(game.mr_eleven, Some("player_1".to_string()));
        assert_eq!(game.players[0].outcome, RollResultNote::Winner);
        assert_eq!(game.players[0].outcome_type, RollResultType::Winner);
        assert_eq!(game.players[0].win_counter, 1);
        assert_eq!(game.players[1].outcome, RollResultNote::SipDrink);
        assert_eq!(game.players[1].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[1].win_counter, 0);
        Ok(())
    }

    #[test]
    fn finishes_instant_loss() -> Result<(), LogicError> {
        let mut game = GameItem::new("game_id", "session_id");
        game.mr_eleven = Some("player_1".to_string());
        game.players = vec![
            PlayerItem {
                player_id: "player_1".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 0,
                rolls: vec![
                    RollItem {
                        dice: vec![
                            DiceItem::new(DiceType::D6, 1),
                            DiceItem::new(DiceType::D6, 1),
                        ],
                    },
                    RollItem {
                        dice: vec![DiceItem::new(DiceType::D4, 1)],
                    },
                ],
                outcome: RollResultNote::FinishDrink,
                outcome_type: RollResultType::Loser,
                finished: true,
            },
            PlayerItem {
                player_id: "player_2".to_string(),
                account_id: None,
                nickname: "nickname".to_string(),
                win_counter: 3,
                rolls: vec![RollItem {
                    dice: vec![
                        DiceItem::new(DiceType::D6, 3),
                        DiceItem::new(DiceType::D6, 3),
                        DiceItem::new(DiceType::D4, 3),
                    ],
                }],
                outcome: RollResultNote::Shower,
                outcome_type: RollResultType::Loser,
                finished: true,
            },
        ];
        handler(&mut game)?;
        assert_eq!(game.round_finished, true);
        assert_eq!(game.mr_eleven, Some("player_1".to_string()));
        assert_eq!(game.players[0].outcome, RollResultNote::FinishDrink);
        assert_eq!(game.players[0].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[0].win_counter, 0);
        assert_eq!(game.players[1].outcome, RollResultNote::Shower);
        assert_eq!(game.players[1].outcome_type, RollResultType::Loser);
        assert_eq!(game.players[1].win_counter, 0);
        Ok(())
    }
}
