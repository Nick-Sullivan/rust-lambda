use crate::attribute_value_parser::{parse_attribute_value, DATETIME_FORMAT};
use crate::{DynamoDbClient, IDynamoDbClient};
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use chrono::{DateTime, Utc};
use domain::errors::LogicError;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, env};

#[derive(Clone, Debug, PartialEq)]
pub enum GameAction {
    CreateGame,
    JoinGame,
    LeaveGame,
    NewRound,
    RollDice,
    StartSpectating,
    StopSpectating,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D10Percentile,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RollResultType {
    #[serde(rename = "0")]
    None,
    #[serde(rename = "1")]
    Loser,
    #[serde(rename = "2")]
    NoChange,
    #[serde(rename = "3")]
    Winner,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RollResultNote {
    #[serde(rename = "")]
    None,
    #[serde(rename = "DUAL_WIELD")]
    DualWield,
    #[serde(rename = "HEAD_ON_TABLE")]
    HeadOnTable,
    #[serde(rename = "FINISH_DRINK")]
    FinishDrink,
    #[serde(rename = "POOL")]
    Pool,
    #[serde(rename = "SIP_DRINK")]
    SipDrink,
    #[serde(rename = "SHOWER")]
    Shower,
    #[serde(rename = "THREE_WAY_TIE")]
    ThreeWayTie,
    #[serde(rename = "TIE")]
    Tie,
    #[serde(rename = "UH_OH")]
    UhOh,
    #[serde(rename = "WINNER")]
    Winner,
    #[serde(rename = "WISH_PURCHASE")]
    WishPurchase,
    #[serde(rename = "COCKRING_HANDS")]
    CockringHands,
}

impl GameAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameAction::CreateGame => "CREATE_GAME",
            GameAction::JoinGame => "JOIN_GAME",
            GameAction::LeaveGame => "LEAVE_GAME",
            GameAction::NewRound => "NEW_ROUND",
            GameAction::RollDice => "ROLL_DICE",
            GameAction::StartSpectating => "START_SPECTATING",
            GameAction::StopSpectating => "STOP_SPECTATING",
        }
    }

    fn from_str(s: &str) -> Result<Self, LogicError> {
        match s {
            "CREATE_GAME" => Ok(GameAction::CreateGame),
            "JOIN_GAME" => Ok(GameAction::JoinGame),
            "LEAVE_GAME" => Ok(GameAction::LeaveGame),
            "NEW_ROUND" => Ok(GameAction::NewRound),
            "ROLL_DICE" => Ok(GameAction::RollDice),
            "START_SPECTATING" => Ok(GameAction::StartSpectating),
            "STOP_SPECTATING" => Ok(GameAction::StopSpectating),
            _ => Err(LogicError::DeserializationError(
                "Invalid game action".to_string(),
            )),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct DiceItem {
    #[serde(rename = "id")]
    pub dice_type: DiceType,
    pub value: i32,
    pub is_death_dice: bool,
}
impl DiceItem {
    pub fn new(dice_type: DiceType, value: i32) -> Self {
        DiceItem {
            dice_type,
            value,
            is_death_dice: false,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RollItem {
    pub dice: Vec<DiceItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RollResultItem {
    pub note: RollResultNote,
    #[serde(rename = "type")]
    pub result_type: RollResultType,
    pub turn_finished: bool,
}

impl RollResultItem {
    pub fn new(note: RollResultNote, result_type: RollResultType, turn_finished: bool) -> Self {
        RollResultItem {
            note,
            result_type,
            turn_finished,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PlayerItem {
    pub player_id: String,
    pub account_id: Option<String>,
    pub nickname: String,
    pub win_counter: i32,
    pub finished: bool,
    pub outcome: RollResultNote,
    pub outcome_type: RollResultType,
    pub rolls: Vec<RollItem>,
    // pub connection_status: ConnectionStatus,
}

impl PlayerItem {
    pub fn new(player_id: &str, account_id: &Option<String>, nickname: &str) -> Self {
        PlayerItem {
            player_id: player_id.to_string(),
            account_id: account_id.clone(),
            nickname: nickname.to_string(),
            win_counter: 0,
            rolls: Vec::new(),
            outcome: RollResultNote::None,
            outcome_type: RollResultType::None,
            finished: false,
        }
    }
    pub fn vec_from_string(json_str: &str) -> Result<Vec<Self>, LogicError> {
        serde_json::from_str(json_str).map_err(|e| LogicError::DeserializationError(e.to_string()))
    }
    pub fn vec_to_string(players: &Vec<Self>) -> Result<String, LogicError> {
        serde_json::to_string(players).map_err(|e| LogicError::SerializationError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct GameItem {
    pub game_id: String,
    pub modified_action: GameAction,
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
    pub mr_eleven: Option<String>,
    pub players: Vec<PlayerItem>,
    pub round_finished: bool,
    // pub round_id: i32,
    // pub spectators: Vec<Spectator>,
    pub version: i32,
}

impl GameItem {
    pub fn new(game_id: &str, session_id: &str) -> Self {
        GameItem {
            game_id: game_id.to_string(),
            modified_action: GameAction::CreateGame,
            modified_at: Utc::now(),
            modified_by: session_id.to_string(),
            mr_eleven: None,
            players: Vec::new(),
            round_finished: false,
            version: 0,
        }
    }

    pub async fn from_db(game_id: &str, db: &DynamoDbClient) -> Result<Self, LogicError> {
        let transaction = Self::get(game_id)?;
        let output = db.read_single(transaction).await?;
        let attribute = output
            .item
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
        let item = Self::from_map(&attribute)?;
        Ok(item)
    }

    pub fn from_map(hash_map: &HashMap<String, AttributeValue>) -> Result<Self, LogicError> {
        let game_id = parse_attribute_value::<String>(hash_map.get("id"))?;
        let modified_action = GameAction::from_str(&parse_attribute_value::<String>(
            hash_map.get("modified_action"),
        )?)?;
        let modified_at = parse_attribute_value::<DateTime<Utc>>(hash_map.get("modified_at"))?;
        let modified_by = parse_attribute_value::<String>(hash_map.get("modified_by"))?;
        let mr_eleven = parse_attribute_value::<Option<String>>(hash_map.get("mr_eleven"))?;
        let players_str = parse_attribute_value::<String>(hash_map.get("players"))?;
        let players = PlayerItem::vec_from_string(&players_str)?;
        let round_finished = parse_attribute_value::<bool>(hash_map.get("round_finished"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;

        let item = GameItem {
            game_id,
            modified_action,
            modified_at,
            modified_by,
            mr_eleven,
            players,
            round_finished,
            version,
        };
        Ok(item)
    }

    fn get_table_name() -> String {
        env::var("GAME_TABLE_NAME").unwrap_or_else(|_| "".to_string())
    }

    pub fn create_game_code() -> String {
        let mut rng = rand::thread_rng();
        let game_code: String = (0..4)
            .map(|_| rng.sample(Alphanumeric) as char)
            .map(|c| c.to_ascii_uppercase())
            .collect();
        game_code
    }

    pub fn get(game_id: &str) -> Result<TransactGetItem, LogicError> {
        let get_item = Get::builder()
            .table_name(Self::get_table_name())
            .key("id", AttributeValue::S(game_id.to_string()))
            .build()
            .map_err(|e| LogicError::GetItemError(e.to_string()))?;
        let transaction_item = TransactGetItem::builder().get(get_item).build();
        Ok(transaction_item)
    }

    pub fn save(&self) -> Result<TransactWriteItem, LogicError> {
        let put_item = Put::builder()
            .table_name(Self::get_table_name())
            .item("id", AttributeValue::S(self.game_id.clone()))
            .item(
                "modified_action",
                AttributeValue::S(self.modified_action.as_str().to_string()),
            )
            .item(
                "modified_at",
                AttributeValue::S(self.modified_at.format(DATETIME_FORMAT).to_string()),
            )
            .item("modified_by", AttributeValue::S(self.modified_by.clone()))
            .item(
                "players",
                AttributeValue::S(PlayerItem::vec_to_string(&self.players)?),
            )
            .item("version", AttributeValue::N(self.version.to_string()))
            .item("round_finished", AttributeValue::Bool(self.round_finished));

        let put_item = match self.mr_eleven {
            Some(ref mr_eleven) => {
                put_item.item("mr_eleven", AttributeValue::S(mr_eleven.to_string()))
            }
            None => put_item,
        };

        let old_version = self.version - 1;
        let put_item = if old_version < 0 {
            put_item.condition_expression("attribute_not_exists(id)")
        } else {
            put_item
                .condition_expression("version = :old_version")
                .expression_attribute_values(
                    ":old_version",
                    AttributeValue::N(old_version.to_string()),
                )
        };
        let put_item = put_item
            .build()
            .map_err(|e| LogicError::UpdateItemError(e.to_string()))?;
        let transaction_item = TransactWriteItem::builder().put(put_item).build();
        Ok(transaction_item)
    }

    pub fn delete(&self) -> Result<TransactWriteItem, LogicError> {
        let delete_item = aws_sdk_dynamodb::types::Delete::builder()
            .table_name(Self::get_table_name())
            .key("id", AttributeValue::S(self.game_id.to_string()))
            .build()
            .map_err(|e| LogicError::DeleteItemError(e.to_string()))?;
        let transaction_item = TransactWriteItem::builder().delete(delete_item).build();
        Ok(transaction_item)
    }
}
