use crate::domain::errors::LogicError;
use crate::storage::attribute_value_parser::parse_attribute_value;
use crate::storage::dynamodb_client::{DynamoDbClient, IDynamoDbClient};
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use std::{collections::HashMap, env};

#[derive(Clone)]
pub struct SessionItem {
    pub session_id: String,
    pub connection_id: String,
    pub version: i32,
}

impl SessionItem {
    pub fn new(session_id: &str, connection_id: &str) -> Self {
        SessionItem {
            session_id: session_id.to_string(),
            connection_id: connection_id.to_string(),
            version: 0,
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
        let session_id = parse_attribute_value::<String>(hash_map.get("id"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;
        let item = SessionItem {
            connection_id,
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
            .item("version", AttributeValue::N(self.version.to_string()));

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
