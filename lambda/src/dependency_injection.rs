#[cfg(not(test))]
mod injections {
    pub use crate::notifier::notifier_cloud::Notifier;
    pub use crate::storage::database_cloud::Database;
    pub use crate::storage::dynamodb_client_cloud::DynamoDbClient;
}
#[cfg(test)]
mod injections {
    pub use crate::notifier::notifier_local::Notifier;
    pub use crate::storage::database_local::Database;
    pub use crate::storage::dynamodb_client_local::DynamoDbClient;
}
use injections::{Database, DynamoDbClient, Notifier};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();
static DYNAMODB_CLIENT: OnceCell<Arc<Mutex<DynamoDbClient>>> = OnceCell::const_new();
static NOTIFIER: OnceCell<Arc<Notifier>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init_database).await.clone()
}

pub async fn get_notifier() -> Arc<Notifier> {
    NOTIFIER.get_or_init(init_notifier).await.clone()
}

pub async fn get_dynamodb_client() -> Arc<Mutex<DynamoDbClient>> {
    DYNAMODB_CLIENT
        .get_or_init(init_dynamodb_client)
        .await
        .clone()
}

async fn init_database() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}

async fn init_notifier() -> Arc<Notifier> {
    let notifier = Notifier::new().await;
    Arc::new(notifier)
}

async fn init_dynamodb_client() -> Arc<Mutex<DynamoDbClient>> {
    let client = DynamoDbClient::new().await;
    Arc::new(Mutex::new(client))
}
