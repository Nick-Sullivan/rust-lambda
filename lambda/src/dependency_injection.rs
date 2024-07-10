use crate::storage;
use std::sync::Arc;
#[cfg(not(test))]
use storage::database_cloud::Database;
#[cfg(test)]
use storage::database_local::Database;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init_database).await.clone()
}

async fn init_database() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}
