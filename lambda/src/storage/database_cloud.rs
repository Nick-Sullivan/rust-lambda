use crate::domain::errors::DatabaseError;
use crate::storage::database::{INameDatabase, NameCount};
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{config::Region, types::AttributeValue, Client};
use std::env;

pub struct Database {
    client: Client,
    table_name: String,
}

#[cfg_attr(test, allow(unused))]
impl Database {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        Database { client, table_name }
    }
}

impl INameDatabase for Database {
    async fn get(&self, name: &str) -> Result<NameCount, DatabaseError> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(name.to_string()))
            .send()
            .await;

        let item = match resp {
            Ok(resp) => resp.item,
            Err(_) => return Err(DatabaseError::NotFound),
        };

        let item = match item {
            Some(item) => item,
            None => {
                return Ok(NameCount {
                    name: name.to_string(),
                    count: 0,
                })
            }
        };

        let count = item.get("count_col").ok_or(DatabaseError::NotFound)?;
        let count_val = match count {
            AttributeValue::N(count_val) => count_val,
            _ => return Err(DatabaseError::NotFound),
        };

        let count = count_val.parse::<i32>().unwrap_or(0);
        Ok(NameCount {
            name: name.to_string(),
            count,
        })
    }

    async fn save(&mut self, item: &NameCount) -> Result<(), DatabaseError> {
        let resp = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("name", AttributeValue::S(item.name.clone()))
            .update_expression("SET count_col = :count_col")
            .expression_attribute_values(":count_col", AttributeValue::N(item.count.to_string()))
            .send()
            .await;
        match resp {
            Ok(_) => Ok(()),
            Err(cause) => Err(DatabaseError::ConnectionError(cause.to_string())),
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
