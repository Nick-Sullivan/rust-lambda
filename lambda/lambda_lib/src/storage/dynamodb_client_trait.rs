use crate::domain::errors::LogicError;
use aws_sdk_dynamodb::types::{ItemResponse, TransactGetItem, TransactWriteItem};

pub trait IDynamoDbClient {
    async fn read_single(&self, item: TransactGetItem) -> Result<ItemResponse, LogicError>;
    async fn write(&self, items: Vec<TransactWriteItem>) -> Result<(), LogicError>;
    async fn write_single(&self, item: TransactWriteItem) -> Result<(), LogicError>;
}
