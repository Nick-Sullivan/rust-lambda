use super::attribute_value_parser::parse_attribute_value;
use crate::domain::errors::LogicError;
use crate::domain::utils::single;
use crate::storage::dynamodb_client::IDynamoDbClient;
use aws_sdk_dynamodb::operation::transact_get_items::builders::TransactGetItemsOutputBuilder;
use aws_sdk_dynamodb::types::{
    AttributeValue, Delete, ItemResponse, Put, TransactGetItem, TransactWriteItem,
};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct FakeItem {
    pub hash_map: HashMap<String, AttributeValue>,
}

pub struct DynamoDbClient {
    game_table: RwLock<HashMap<String, FakeItem>>,
    websocket_table: RwLock<HashMap<String, FakeItem>>,
}

#[cfg_attr(not(test), allow(unused))]
impl DynamoDbClient {
    pub async fn new() -> Self {
        let game_table = RwLock::new(HashMap::new());
        let websocket_table = RwLock::new(HashMap::new());
        DynamoDbClient {
            game_table,
            websocket_table,
        }
    }

    fn get_table(&self, table_name: &str) -> &RwLock<HashMap<String, FakeItem>> {
        match table_name {
            "GAME" => &self.game_table,
            "WEBSOCKET" => &self.websocket_table,
            _ => panic!("Unrecognised table {:?}", table_name),
        }
    }

    fn get_primary_key(&self, table_name: &str) -> &str {
        match table_name {
            "GAME" => "id",
            "WEBSOCKET" => "connection_id",
            _ => panic!("Unrecognised table"),
        }
    }

    fn write_put(&self, put: Put) -> Result<(), LogicError> {
        let table = self.get_table(&put.table_name);
        let primary_key_column = self.get_primary_key(&put.table_name);
        let primary_key = parse_attribute_value::<String>(put.item.get(primary_key_column))?;
        let item = FakeItem {
            hash_map: put.item.clone(),
        };
        let mut hash_map = table.write().unwrap();
        if put.condition_expression.is_some() {
            let existing_item = hash_map.get(&primary_key);
            self.check_put_condition(put, &existing_item)?;
        }
        hash_map.insert(primary_key.to_string(), item);
        Ok(())
    }

    fn write_delete(&self, delete: Delete) -> Result<(), LogicError> {
        let table = self.get_table(&delete.table_name);
        let primary_key_column = self.get_primary_key(&delete.table_name);
        let primary_key = parse_attribute_value::<String>(delete.key.get(primary_key_column))?;
        let mut hash_map = table.write().unwrap();
        if delete.condition_expression.is_some() {
            let existing_item = hash_map.get(&primary_key);
            self.check_delete_condition(delete, &existing_item)?;
        }
        hash_map.remove(&primary_key.to_string());
        Ok(())
    }

    fn check_put_condition(
        &self,
        put: Put,
        existing_item: &Option<&FakeItem>,
    ) -> Result<(), LogicError> {
        let expression = put
            .condition_expression
            .ok_or(LogicError::ConditionalCheckFailed(
                "No condition".to_string(),
            ))?;

        let must_be_new = expression.starts_with("attribute_not_exists");
        match (existing_item, must_be_new) {
            (Some(_), true) => {
                return Err(LogicError::UpdateItemError(
                    "Item already exists".to_string(),
                ));
            }
            (Some(existing_item), false) => {
                let actual_version =
                    parse_attribute_value::<i32>(existing_item.hash_map.get("version"))?;
                let new_version = parse_attribute_value::<i32>(put.item.get("version"))?;
                if new_version != actual_version + 1 {
                    return Err(LogicError::UpdateItemError("Version mismatch".to_string()));
                }
            }
            (None, false) => {
                return Err(LogicError::UpdateItemError(
                    "Item does not exist".to_string(),
                ));
            }
            _ => {}
        }
        return Ok(());
    }

    fn check_delete_condition(
        &self,
        delete: Delete,
        existing_item: &Option<&FakeItem>,
    ) -> Result<(), LogicError> {
        let _ = delete
            .condition_expression
            .ok_or(LogicError::ConditionalCheckFailed(
                "No condition".to_string(),
            ))?;
        let attributes = delete
            .expression_attribute_values
            .ok_or(LogicError::DeleteItemError(
                "No expression values".to_string(),
            ))?;
        let expected_version =
            parse_attribute_value::<i32>(attributes.get(&":old_version".to_string()))?;

        match existing_item {
            Some(existing_item) => {
                let actual_version =
                    parse_attribute_value::<i32>(existing_item.hash_map.get("version"))?;
                if expected_version != actual_version {
                    return Err(LogicError::UpdateItemError("Version mismatch".to_string()));
                }
            }
            None => {
                return Err(LogicError::UpdateItemError(
                    "Item does not exist".to_string(),
                ));
            }
        }
        return Ok(());
    }
}

impl IDynamoDbClient for DynamoDbClient {
    async fn read(&self, item: TransactGetItem) -> Result<ItemResponse, LogicError> {
        let get = item.get.ok_or(LogicError::GetItemError(
            "Only Gets are supported".to_string(),
        ))?;
        let table = self.get_table(&get.table_name);
        let primary_key_column = self.get_primary_key(&get.table_name);
        let hash_map = table.read().unwrap();
        let primary_key = parse_attribute_value::<String>(get.key.get(primary_key_column))?;
        let item = hash_map
            .get(&primary_key)
            .ok_or(LogicError::GetItemError("Item not found".to_string()))?;

        let item_response = ItemResponse::builder()
            .set_item(Some(item.hash_map.clone()))
            .build();
        let output = TransactGetItemsOutputBuilder::default()
            .responses(item_response)
            .build();
        let items = output
            .responses
            .ok_or(LogicError::GetItemError("No response".to_string()))?;
        let item = single(items).map_err(|e| LogicError::GetItemError(e.to_string()))?;
        Ok(item)
    }

    async fn write(&self, items: Vec<TransactWriteItem>) -> Result<(), LogicError> {
        for item in items {
            if let Some(put) = item.put {
                self.write_put(put)?;
            } else if let Some(delete) = item.delete {
                self.write_delete(delete)?;
            } else {
                return Err(LogicError::UpdateItemError(
                    "Only Put/Delete is supported".to_string(),
                ));
            }
        }
        Ok(())
    }
}
