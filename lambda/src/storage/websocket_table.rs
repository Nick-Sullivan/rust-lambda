use super::attribute_value_parser::parse_attribute_value;
use crate::domain::errors::LogicError;
use aws_sdk_dynamodb::types::{AttributeValue, Get, Put, TransactGetItem, TransactWriteItem};
use std::{collections::HashMap, env};

#[derive(Clone)]
pub struct WebsocketItem {
    pub connection_id: String,
    pub session_id: Option<String>,
    pub version: i32,
    // modified_at: datetime = None
}

impl WebsocketItem {
    pub fn new(connection_id: &str) -> Self {
        WebsocketItem {
            connection_id: connection_id.to_string(),
            session_id: None,
            version: 0,
        }
    }

    pub fn from_map(hash_map: &HashMap<String, AttributeValue>) -> Result<Self, LogicError> {
        let connection_id = parse_attribute_value::<String>(hash_map.get("connection_id"))?;
        let session_id = parse_attribute_value::<Option<String>>(hash_map.get("session_id"))?;
        let version = parse_attribute_value::<i32>(hash_map.get("version"))?;
        let item = WebsocketItem {
            connection_id,
            session_id,
            version,
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
            .item("version", AttributeValue::N(self.version.to_string()));
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
