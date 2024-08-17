use crate::database::Database;
use crate::dynamodb_client::DynamoDbClient;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();
static DYNAMODB_CLIENT: OnceCell<Arc<DynamoDbClient>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init_database).await.clone()
}

pub async fn get_dynamodb_client() -> Arc<DynamoDbClient> {
    DYNAMODB_CLIENT
        .get_or_init(init_dynamodb_client)
        .await
        .clone()
}

async fn init_database() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}

async fn init_dynamodb_client() -> Arc<DynamoDbClient> {
    let client = DynamoDbClient::new().await;
    Arc::new(client)
}
