use crate::domain::errors::LogicError;
use std::collections::HashMap;

use super::websocket_table::{IWebsocketTable, WebsocketItem};

pub struct WebsocketTable {
    connections: HashMap<String, WebsocketItem>,
}

#[cfg_attr(not(test), allow(unused))]
impl WebsocketTable {
    pub async fn new() -> Self {
        let connections = HashMap::new();
        WebsocketTable { connections }
    }
    async fn create(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        if self.connections.contains_key(&item.connection_id) {
            return Err(LogicError::UpdateItemError(item.connection_id.clone()));
        }
        self.connections
            .insert(item.connection_id.clone(), item.clone());
        Ok(())
    }

    async fn update(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        let existing_item = self.get(&item.connection_id).await?;
        if existing_item.version != item.version - 1 {
            return Err(LogicError::UpdateItemError(item.connection_id.clone()));
        }
        self.connections
            .insert(item.connection_id.clone(), item.clone());
        Ok(())
    }
}

impl IWebsocketTable for WebsocketTable {
    async fn get(&self, id: &str) -> Result<WebsocketItem, LogicError> {
        let item = self.connections.get(id);
        match item {
            Some(item) => Ok(item.clone()),
            None => return Err(LogicError::GetItemError(id.to_string())),
        }
    }

    async fn save(&mut self, item: &WebsocketItem) -> Result<(), LogicError> {
        if item.version == 0 {
            self.create(item).await
        } else {
            self.update(item).await
        }
    }

    async fn clear(&mut self, id: &str) -> Result<(), LogicError> {
        self.connections.remove(id);
        Ok(())
    }
}
