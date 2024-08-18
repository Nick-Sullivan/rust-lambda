use crate::attribute_value_parser::{parse_attribute_value, DATETIME_FORMAT};
use crate::dynamodb_client::{DynamoDbClient, IDynamoDbClient};
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use chrono::{DateTime, Utc};
use domain::errors::LogicError;
use rand::distributions::Alphanumeric;
use rand::Rng;
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

#[derive(Clone)]
pub struct GameItem {
    pub game_id: String,
    pub modified_action: GameAction,
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
    pub mr_eleven: Option<String>,
    // pub players: Vec<Player>,
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
        let round_finished = parse_attribute_value::<bool>(hash_map.get("round_finished"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;

        let item = GameItem {
            game_id,
            modified_action,
            modified_at,
            modified_by,
            mr_eleven,
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
}
