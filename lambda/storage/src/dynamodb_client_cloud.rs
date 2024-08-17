use crate::dynamodb_client::IDynamoDbClient;
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::types::{ItemResponse, TransactGetItem, TransactWriteItem};
use aws_sdk_dynamodb::{config::Region, Client};
use domain::errors::LogicError;
use domain::utils;
use std::env;

pub struct DynamoDbClient {
    client: Client,
}

#[cfg_attr(test, allow(unused))]
impl DynamoDbClient {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        DynamoDbClient { client }
    }
}

impl IDynamoDbClient for DynamoDbClient {
    async fn read_single(&self, item: TransactGetItem) -> Result<ItemResponse, LogicError> {
        let result = self
            .client
            .transact_get_items()
            .transact_items(item)
            .send()
            .await
            .map_err(|e| LogicError::GetItemError(e.to_string()))?;
        let items = result
            .responses
            .ok_or(LogicError::GetItemError("No response".to_string()))?;
        let item = utils::single(items).map_err(|e| LogicError::GetItemError(e.to_string()))?;
        Ok(item)
    }

    async fn write(&self, items: Vec<TransactWriteItem>) -> Result<(), LogicError> {
        let result = self
            .client
            .transact_write_items()
            .set_transact_items(Some(items))
            .send()
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(LogicError::UpdateItemError(e.to_string())),
        }
    }

    async fn write_single(&self, item: TransactWriteItem) -> Result<(), LogicError> {
        self.write(vec![item]).await
    }
}
