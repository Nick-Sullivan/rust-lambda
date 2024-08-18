use crate::database::{INameDatabase, NameCount};
use aws_config::meta::region::RegionProviderChain;
use aws_config::{self, BehaviorVersion};
use aws_sdk_dynamodb::{
    config::Region, operation::update_item::UpdateItemError, types::AttributeValue, Client,
};
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use domain::errors::LogicError;
use std::collections::HashMap;
use std::env;

pub struct Database {
    client: Client,
    table_name: String,
}

#[cfg_attr(feature = "in_memory", allow(unused))]
impl Database {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        Database { client, table_name }
    }

    async fn create(&mut self, item: &NameCount) -> Result<(), LogicError> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(item.name.clone()))
            .update_expression("SET count_col = :count_col, version = :version")
            .condition_expression("attribute_not_exists(version)")
            .expression_attribute_values(":count_col", AttributeValue::N(item.count.to_string()))
            .expression_attribute_values(":version", AttributeValue::N(item.version.to_string()))
            .send()
            .await
            .map_err(|e| self.parse_update_error(e))?;
        Ok(())
    }

    async fn update(&mut self, item: &NameCount) -> Result<(), LogicError> {
        let old_version = item.version - 1;
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(item.name.clone()))
            .update_expression("SET count_col = :count_col, version = :version")
            .condition_expression("version = :old_version")
            .expression_attribute_values(":count_col", AttributeValue::N(item.count.to_string()))
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

    fn parse_item(&self, item: &HashMap<String, AttributeValue>) -> Result<NameCount, LogicError> {
        let count: i32 = Self::get_parsed_value(item, "count_col")?;
        let version: i32 = Self::get_parsed_value(item, "version")?;
        let name: String = Self::get_parsed_value(item, "name")?;
        Ok(NameCount {
            name: name.to_string(),
            count,
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

impl INameDatabase for Database {
    async fn get(&self, name: &str) -> Result<NameCount, LogicError> {
        let response = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::GetItemError(err.to_string()))?;

        let item = match response.item {
            Some(item) => self.parse_item(&item)?,
            None => NameCount::new(name),
        };
        Ok(item)
    }

    async fn save(&mut self, item: &NameCount) -> Result<(), LogicError> {
        if item.version == 1 {
            self.create(item).await
        } else {
            self.update(item).await
        }
    }

    async fn clear(&mut self, name: &str) -> Result<(), LogicError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .send()
            .await
            .map_err(|err| LogicError::DeleteItemError(err.to_string()))?;
        Ok(())
    }
}
