use crate::domain::errors::LogicError;
use crate::storage::websocket_table::{IWebsocketTable, WebsocketItem};
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    config::Region, operation::update_item::UpdateItemError, types::AttributeValue, Client,
};
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use std::collections::HashMap;
use std::env;

pub struct WebsocketTable {
    client: Client,
    table_name: String,
}

#[cfg_attr(test, allow(unused))]
impl WebsocketTable {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let table_name = env::var("WEBSOCKET_TABLE_NAME").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        WebsocketTable { client, table_name }
    }

    async fn create(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        println!("DB: Creating websocket item");
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key(
                "connection_id",
                AttributeValue::S(item.connection_id.clone()),
            )
            .update_expression("SET version = :version")
            .condition_expression("attribute_not_exists(version)")
            .expression_attribute_values(":version", AttributeValue::N(item.version.to_string()))
            .send()
            .await
            .map_err(|e| self.parse_update_error(e))?;
        println!("DB: Item created");
        Ok(())
    }

    async fn update(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        let old_version = item.version - 1;
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key(
                "connection_id",
                AttributeValue::S(item.connection_id.clone()),
            )
            .update_expression("SET version = :version")
            .condition_expression("version = :old_version")
            .expression_attribute_values(":version", AttributeValue::N(item.version.to_string()))
            .expression_attribute_values(":old_version", AttributeValue::N(old_version.to_string()))
            .send()
            .await
            .map_err(|e| self.parse_update_error(e))?;
        Ok(())
    }

    fn parse_update_error(&self, err: SdkError<UpdateItemError, Response>) -> LogicError {
        match err {
            SdkError::ServiceError(se) => match se.err() {
                UpdateItemError::ConditionalCheckFailedException(err) => {
                    LogicError::ConditionalCheckFailed(err.to_string())
                }
                _ => LogicError::UpdateItemError(se.err().to_string()),
            },
            _ => LogicError::UpdateItemError(err.to_string()),
        }
    }

    fn parse_item(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<WebsocketItem, LogicError> {
        let version: i32 = Self::get_parsed_value(item, "version")?;
        let connection_id: String = Self::get_parsed_value(item, "connection_id")?;
        Ok(WebsocketItem {
            connection_id: connection_id.to_string(),
            version,
        })
    }

    fn get_parsed_value<T: std::str::FromStr>(
        item: &HashMap<String, AttributeValue>,
        key: &str,
    ) -> Result<T, LogicError> {
        let value = item.get(key).ok_or(LogicError::ParseItemError(
            "Attribute {key} not found".to_string(),
        ))?;
        match value {
            AttributeValue::N(val) => val
                .parse::<T>()
                .map_err(|_| LogicError::ParseItemError("Unable to parse {key}".to_string())),
            AttributeValue::S(val) => val
                .parse::<T>()
                .map_err(|_| LogicError::ParseItemError("Unable to parse {key}".to_string())),
            _ => Err(LogicError::ParseItemError(
                "Unsupported type to parse argument {key}".to_string(),
            )),
        }
    }
}

impl IWebsocketTable for WebsocketTable {
    async fn get(&self, id: &str) -> Result<WebsocketItem, LogicError> {
        let response = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("connection_id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::GetItemError(err.to_string()))?;

        let item = match response.item {
            Some(item) => self.parse_item(&item)?,
            None => return Err(LogicError::GetItemError(id.to_string())),
        };
        Ok(item)
    }

    async fn save(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        if item.version == 0 {
            self.create(item).await
        } else {
            self.update(item).await
        }
    }

    async fn clear(&mut self, id: &str) -> Result<(), LogicError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("connection_id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::DeleteItemError(err.to_string()))?;
        Ok(())
    }
}
