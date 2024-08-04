use crate::domain::errors::LogicError;
use crate::storage::session_table::{ISessionTable, SessionItem};
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    config::Region, operation::update_item::UpdateItemError, types::AttributeValue, Client,
};
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use std::collections::HashMap;
use std::env;

pub struct SessionTable {
    client: Client,
    table_name: String,
}

#[cfg_attr(test, allow(unused))]
impl SessionTable {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let table_name = env::var("GAME_TABLE_NAME").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        SessionTable { client, table_name }
    }

    async fn create(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        println!("DB creating session_id: {}", item.session_id);
        println!("DB creating version: {}", item.version);
        println!("DB creating connection_id: {}", item.connection_id);
        println!("DB table: {}", self.table_name);
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(item.session_id.clone()))
            .update_expression("SET connection_id = :connection_id, version = :version")
            .condition_expression("attribute_not_exists(version)")
            .expression_attribute_values(
                ":connection_id",
                AttributeValue::S(item.connection_id.to_string()),
            )
            .expression_attribute_values(":version", AttributeValue::N(item.version.to_string()))
            .send()
            .await
            .map_err(|e| self.parse_update_error(e))?;
        println!("DB item created");
        Ok(())
    }

    async fn update(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        let old_version = item.version - 1;
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(item.connection_id.clone()))
            .update_expression("SET connection_id = :connection_id, version = :version")
            .condition_expression("version = :old_version")
            .expression_attribute_values(
                ":connection_id",
                AttributeValue::N(item.connection_id.to_string()),
            )
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
    ) -> Result<SessionItem, LogicError> {
        let version: i32 = Self::get_parsed_value(item, "version")?;
        let id: String = Self::get_parsed_value(item, "id")?;
        let connection_id: String = Self::get_parsed_value(item, "connection_id")?;
        Ok(SessionItem {
            session_id: id.to_string(),
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

impl ISessionTable for SessionTable {
    async fn get(&self, session_id: &str) -> Result<SessionItem, LogicError> {
        let response = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(session_id.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::GetItemError(err.to_string()))?;

        let item = match response.item {
            Some(item) => self.parse_item(&item)?,
            None => return Err(LogicError::GetItemError(session_id.to_string())),
        };
        Ok(item)
    }

    async fn save(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        if item.version == 0 {
            self.create(item).await
        } else {
            self.update(item).await
        }
    }

    async fn clear(&mut self, session_id: &str) -> Result<(), LogicError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("session_id", AttributeValue::S(session_id.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::DeleteItemError(err.to_string()))?;
        Ok(())
    }
}
