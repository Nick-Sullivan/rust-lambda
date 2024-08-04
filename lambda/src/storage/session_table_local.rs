use crate::domain::errors::LogicError;
use std::collections::HashMap;

use super::session_table::{ISessionTable, SessionItem};

pub struct SessionTable {
    sessions: HashMap<String, SessionItem>,
}

#[cfg_attr(not(test), allow(unused))]
impl SessionTable {
    pub async fn new() -> Self {
        let sessions = HashMap::new();
        SessionTable { sessions }
    }
    async fn create(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        if self.sessions.contains_key(&item.session_id) {
            return Err(LogicError::UpdateItemError(item.session_id.clone()));
        }
        self.sessions.insert(item.session_id.clone(), item.clone());
        Ok(())
    }

    async fn update(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        let existing_item = self.get(&item.session_id).await?;
        if existing_item.version != item.version - 1 {
            return Err(LogicError::UpdateItemError(item.session_id.clone()));
        }
        self.sessions.insert(item.session_id.clone(), item.clone());
        Ok(())
    }
}

impl ISessionTable for SessionTable {
    async fn get(&self, session_id: &str) -> Result<SessionItem, LogicError> {
        let item = self.sessions.get(session_id);
        match item {
            Some(item) => Ok(item.clone()),
            None => return Err(LogicError::GetItemError(session_id.to_string())),
        }
    }

    async fn save(&mut self, item: &SessionItem) -> Result<(), LogicError> {
        if item.version == 0 {
            self.create(item).await
        } else {
            self.update(item).await
        }
    }

    async fn clear(&mut self, session_id: &str) -> Result<(), LogicError> {
        self.sessions.remove(session_id);
        Ok(())
    }
}
