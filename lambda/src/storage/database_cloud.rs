use crate::domain::errors::DatabaseError;
use crate::storage::database::INameDatabase;
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{config::Region, types::AttributeValue, Client};
use std::env;

pub struct NameCounter {
    client: Client,
    table_name: String,
}

#[cfg_attr(test, allow(unused))]
impl NameCounter {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        NameCounter { client, table_name }
    }
}

impl INameDatabase for NameCounter {
    async fn increment(&mut self, name: &str) -> Result<(), DatabaseError> {
        let resp = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .expression_attribute_values(":start", AttributeValue::N("0".to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .update_expression("SET count_col = if_not_exists(count_col, :start) + :inc")
            .send()
            .await;
        match resp {
            Ok(_) => Ok(()),
            Err(cause) => Err(DatabaseError::ConnectionError(cause.to_string())),
        }
    }

    async fn get_count(&self, name: &str) -> Result<i32, DatabaseError> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .send()
            .await;
        match resp {
            Ok(resp) => {
                let item = resp.item();
                if item.is_none() {
                    return Ok(0);
                }
                let item = item.unwrap();
                let count = item.get("count_col").ok_or(DatabaseError::NotFound)?;
                match count {
                    AttributeValue::N(count_val) => Ok(count_val.parse::<i32>().unwrap_or(0)),
                    _ => Err(DatabaseError::NotFound),
                }
            }
            Err(_) => return Err(DatabaseError::NotFound),
        }
    }

    async fn clear(&mut self, name: &str) -> Result<(), DatabaseError> {
        let resp = self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .send()
            .await;
        match resp {
            Ok(_) => Ok(()),
            Err(_) => Err(DatabaseError::NotFound),
        }
    }
}
