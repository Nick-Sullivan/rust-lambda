use crate::attribute_value_parser::{parse_attribute_value, DATETIME_FORMAT};
use crate::{DynamoDbClient, IDynamoDbClient};
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use chrono::{DateTime, Utc};
use domain::errors::LogicError;
use std::{collections::HashMap, env};

#[derive(Clone)]
pub struct WebsocketItem {
    pub connection_id: String,
    pub session_id: Option<String>,
    pub version: i32,
    pub modified_at: DateTime<Utc>,
}

impl WebsocketItem {
    pub fn new(connection_id: &str) -> Self {
        WebsocketItem {
            connection_id: connection_id.to_string(),
            session_id: None,
            version: 0,
            modified_at: Utc::now(),
        }
    }

    pub fn new_with_session(connection_id: &str, session_id: &str) -> Self {
        WebsocketItem {
            connection_id: connection_id.to_string(),
            session_id: Some(session_id.to_string()),
            version: 0,
            modified_at: Utc::now(),
        }
    }

    pub async fn from_db(connection_id: &str, db: &DynamoDbClient) -> Result<Self, LogicError> {
        let transaction = Self::get(connection_id)?;
        let output = db.read_single(transaction).await?;
        let attribute = output
            .item
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;
        let item = Self::from_map(&attribute)?;
        Ok(item)
    }

    pub fn from_map(hash_map: &HashMap<String, AttributeValue>) -> Result<Self, LogicError> {
        let connection_id = parse_attribute_value::<String>(hash_map.get("connection_id"))?;
        let session_id = parse_attribute_value::<Option<String>>(hash_map.get("session_id"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;
        let modified_at = parse_attribute_value::<DateTime<Utc>>(hash_map.get("modified_at"))?;
        let item = WebsocketItem {
            connection_id,
            session_id,
            version,
            modified_at,
        };
        Ok(item)
    }

    fn get_table_name() -> String {
        env::var("WEBSOCKET_TABLE_NAME").unwrap_or_else(|_| "".to_string())
    }

    pub fn get(connection_id: &str) -> Result<TransactGetItem, LogicError> {
        let get_item = Get::builder()
            .table_name(Self::get_table_name())
            .key(
                "connection_id",
                AttributeValue::S(connection_id.to_string()),
            )
            .build()
            .map_err(|e| LogicError::GetItemError(e.to_string()))?;
        let transaction_item = TransactGetItem::builder().get(get_item).build();
        Ok(transaction_item)
    }

    pub fn save(&self) -> Result<TransactWriteItem, LogicError> {
        let put_item = Put::builder()
            .table_name(Self::get_table_name())
            .item(
                "connection_id",
                AttributeValue::S(self.connection_id.to_string()),
            )
            .item("version", AttributeValue::N(self.version.to_string()))
            .item(
                "modified_at",
                AttributeValue::S(self.modified_at.format(DATETIME_FORMAT).to_string()),
            );
        let put_item = match self.session_id {
            Some(ref session_id) => {
                put_item.item("session_id", AttributeValue::S(session_id.to_string()))
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
            .key(
                "connection_id",
                AttributeValue::S(self.connection_id.to_string()),
            )
            .build()
            .map_err(|e| LogicError::DeleteItemError(e.to_string()))?;
        let transaction_item = TransactWriteItem::builder().delete(delete_item).build();
        Ok(transaction_item)
    }
}
