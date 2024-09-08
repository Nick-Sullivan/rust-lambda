use crate::Database;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init).await.clone()
}

async fn init() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}
