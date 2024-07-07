use crate::storage;
use std::sync::Arc;
#[cfg(not(test))]
use storage::database_cloud::NameCounter;
#[cfg(test)]
use storage::database_local::NameCounter;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<NameCounter>>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<NameCounter>> {
    DATABASE.get_or_init(init_database).await.clone()
}

async fn init_database() -> Arc<Mutex<NameCounter>> {
    let db = NameCounter::new().await;
    Arc::new(Mutex::new(db))
}