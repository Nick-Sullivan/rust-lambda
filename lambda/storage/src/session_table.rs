use crate::attribute_value_parser::{parse_attribute_value, DATETIME_FORMAT};
use crate::{DynamoDbClient, IDynamoDbClient};
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use chrono::{DateTime, Utc};
use domain::errors::LogicError;
use std::{collections::HashMap, env};

#[derive(Clone, Debug, PartialEq)]
pub enum SessionAction {
    CreateConnection,
    SetNickname,
    JoinGame,
    PendingTimeout,
    Reconnected,
}
impl SessionAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionAction::CreateConnection => "CREATE_CONNECTION",
            SessionAction::SetNickname => "SET_NICKNAME",
            SessionAction::JoinGame => "JOIN_GAME",
            SessionAction::PendingTimeout => "PENDING_TIMEOUT",
            SessionAction::Reconnected => "RECONNECTED",
        }
    }

    fn from_str(s: &str) -> Result<Self, LogicError> {
        match s {
            "CREATE_CONNECTION" => Ok(SessionAction::CreateConnection),
            "SET_NICKNAME" => Ok(SessionAction::SetNickname),
            "JOIN_GAME" => Ok(SessionAction::JoinGame),
            "PENDING_TIMEOUT" => Ok(SessionAction::PendingTimeout),
            "RECONNECTED" => Ok(SessionAction::Reconnected),
            _ => Err(LogicError::DeserializationError(
                "Invalid session action".to_string(),
            )),
        }
    }
}

#[derive(Clone)]
pub struct SessionItem {
    pub account_id: Option<String>,
    pub connection_id: String,
    pub game_id: Option<String>,
    pub modified_at: DateTime<Utc>,
    pub modified_action: SessionAction,
    pub nickname: Option<String>,
    pub session_id: String,
    pub version: i32,
}

impl SessionItem {
    pub fn new(session_id: &str, connection_id: &str) -> Self {
        SessionItem {
            account_id: None,
            connection_id: connection_id.to_string(),
            game_id: None,
            nickname: None,
            modified_at: Utc::now(),
            modified_action: SessionAction::CreateConnection,
            session_id: session_id.to_string(),
            version: 0,
        }
    }

    pub async fn from_db(session_id: &str, db: &DynamoDbClient) -> Result<Self, LogicError> {
        let transaction = Self::get(session_id)?;
        let output = db.read_single(transaction).await?;
        let attribute = output
            .item
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
        let item = Self::from_map(&attribute)?;
        Ok(item)
    }

    pub fn from_map(hash_map: &HashMap<String, AttributeValue>) -> Result<Self, LogicError> {
        let account_id = parse_attribute_value::<Option<String>>(hash_map.get("account_id"))?;
        let connection_id = parse_attribute_value::<String>(hash_map.get("connection_id"))?;
        let game_id = parse_attribute_value::<Option<String>>(hash_map.get("game_id"))?;
        let modified_at = parse_attribute_value::<DateTime<Utc>>(hash_map.get("modified_at"))?;
        let modified_action = SessionAction::from_str(&parse_attribute_value::<String>(
            hash_map.get("modified_action"),
        )?)?;
        let nickname = parse_attribute_value::<Option<String>>(hash_map.get("nickname"))?;
        let session_id = parse_attribute_value::<String>(hash_map.get("id"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;

        let item = SessionItem {
            account_id,
            connection_id,
            game_id,
            modified_at,
            modified_action,
            nickname,
            session_id,
            version,
        };
        Ok(item)
    }

    fn get_table_name() -> String {
        env::var("GAME_TABLE_NAME").unwrap_or_else(|_| "".to_string())
    }

    pub fn get(session_id: &str) -> Result<TransactGetItem, LogicError> {
        let get_item = Get::builder()
            .table_name(Self::get_table_name())
            .key("id", AttributeValue::S(session_id.to_string()))
            .build()
            .map_err(|e| LogicError::GetItemError(e.to_string()))?;
        let transaction_item = TransactGetItem::builder().get(get_item).build();
        Ok(transaction_item)
    }

    pub fn save(&self) -> Result<TransactWriteItem, LogicError> {
        let put_item = Put::builder()
            .table_name(Self::get_table_name())
            .item("id", AttributeValue::S(self.session_id.clone()))
            .item(
                "connection_id",
                AttributeValue::S(self.connection_id.to_string()),
            )
            .item(
                "modified_action",
                AttributeValue::S(self.modified_action.as_str().to_string()),
            )
            .item(
                "modified_at",
                AttributeValue::S(self.modified_at.format(DATETIME_FORMAT).to_string()),
            )
            .item("version", AttributeValue::N(self.version.to_string()));

        let put_item = match self.account_id {
            Some(ref account_id) => {
                put_item.item("account_id", AttributeValue::S(account_id.to_string()))
            }
            None => put_item,
        };

        let put_item = match self.game_id {
            Some(ref game_id) => put_item.item("game_id", AttributeValue::S(game_id.to_string())),
            None => put_item,
        };

        let put_item = match self.nickname {
            Some(ref nickname) => {
                put_item.item("nickname", AttributeValue::S(nickname.to_string()))
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
            .key("id", AttributeValue::S(self.session_id.to_string()))
            .build()
            .map_err(|e| LogicError::DeleteItemError(e.to_string()))?;
        let transaction_item = TransactWriteItem::builder().delete(delete_item).build();
        Ok(transaction_item)
    }
}
