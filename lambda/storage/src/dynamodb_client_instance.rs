use crate::DynamoDbClient;
use std::sync::Arc;
use tokio::sync::OnceCell;

static DYNAMODB_CLIENT: OnceCell<Arc<DynamoDbClient>> = OnceCell::const_new();

pub async fn get() -> Arc<DynamoDbClient> {
    DYNAMODB_CLIENT.get_or_init(init).await.clone()
}

async fn init() -> Arc<DynamoDbClient> {
    let client = DynamoDbClient::new().await;
    Arc::new(client)
}
