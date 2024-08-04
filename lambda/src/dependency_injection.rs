#[cfg(not(test))]
mod injections {
    pub use crate::notifier::notifier_cloud::Notifier;
    pub use crate::storage::database_cloud::Database;
    pub use crate::storage::session_table_cloud::SessionTable;
    pub use crate::storage::websocket_table_cloud::WebsocketTable;
}
#[cfg(test)]
mod injections {
    pub use crate::notifier::notifier_local::Notifier;
    pub use crate::storage::database_local::Database;
    pub use crate::storage::session_table_local::SessionTable;
    pub use crate::storage::websocket_table_local::WebsocketTable;
}
use injections::{Database, Notifier, SessionTable, WebsocketTable};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static DATABASE: OnceCell<Arc<Mutex<Database>>> = OnceCell::const_new();
static SESSION_TABLE: OnceCell<Arc<Mutex<SessionTable>>> = OnceCell::const_new();
static WEBSOCKET_TABLE: OnceCell<Arc<Mutex<WebsocketTable>>> = OnceCell::const_new();
static NOTIFIER: OnceCell<Arc<Notifier>> = OnceCell::const_new();

pub async fn get_database() -> Arc<Mutex<Database>> {
    DATABASE.get_or_init(init_database).await.clone()
}

pub async fn get_notifier() -> Arc<Notifier> {
    NOTIFIER.get_or_init(init_notifier).await.clone()
}

pub async fn get_websocket_table() -> Arc<Mutex<WebsocketTable>> {
    WEBSOCKET_TABLE
        .get_or_init(init_websocket_table)
        .await
        .clone()
}

pub async fn get_session_table() -> Arc<Mutex<SessionTable>> {
    SESSION_TABLE.get_or_init(init_session_table).await.clone()
}

async fn init_database() -> Arc<Mutex<Database>> {
    let db = Database::new().await;
    Arc::new(Mutex::new(db))
}

async fn init_notifier() -> Arc<Notifier> {
    let notifier = Notifier::new().await;
    Arc::new(notifier)
}

async fn init_session_table() -> Arc<Mutex<SessionTable>> {
    let table = SessionTable::new().await;
    Arc::new(Mutex::new(table))
}

async fn init_websocket_table() -> Arc<Mutex<WebsocketTable>> {
    let table = WebsocketTable::new().await;
    Arc::new(Mutex::new(table))
}
