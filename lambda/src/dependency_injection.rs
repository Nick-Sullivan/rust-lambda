#[cfg(not(test))]
mod injections {
    pub use crate::notifier::notifier_cloud::Notifier;
    pub use crate::storage::database_cloud::Database;
}
#[cfg(test)]
mod injections {
    pub use crate::notifier::notifier_local::Notifier;
    pub use crate::storage::database_local::Database;
}
use injections::{Database, Notifier};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();
static NOTIFIER: OnceCell<Arc<Notifier>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init_database).await.clone()
}

pub async fn get_notifier() -> Arc<Notifier> {
    NOTIFIER.get_or_init(init_notifier).await.clone()
}

async fn init_database() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}

async fn init_notifier() -> Arc<Notifier> {
    let notifier = Notifier::new().await;
    Arc::new(notifier)
}
