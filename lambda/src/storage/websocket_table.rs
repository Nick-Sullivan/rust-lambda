use crate::domain::errors::LogicError;

#[derive(Clone)]
pub struct WebsocketItem {
    pub connection_id: String,
    pub version: i32,
}

impl WebsocketItem {
    pub fn new(connection_id: &str) -> Self {
        WebsocketItem {
            connection_id: connection_id.to_string(),
            version: 0,
        }
    }
}
pub trait IWebsocketTable {
    async fn get(&self, connection_id: &str) -> Result<WebsocketItem, LogicError>;
    async fn save(&mut self, item: &WebsocketItem) -> Result<(), LogicError>;
    async fn clear(&mut self, connection_id: &str) -> Result<(), LogicError>;
}
