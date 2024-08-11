use crate::domain::errors::LogicError;
use aws_sdk_dynamodb::types::{ItemResponse, TransactGetItem, TransactWriteItem};

pub trait IDynamoDbClient {
    async fn read(&self, item: TransactGetItem) -> Result<ItemResponse, LogicError>;
    async fn write(&self, items: Vec<TransactWriteItem>) -> Result<(), LogicError>;
}
