use crate::notifier::{INotifier, Message};
use domain::errors::LogicError;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Notifier {
    pub log: RwLock<HashMap<String, Vec<String>>>,
}

#[cfg_attr(not(test), allow(unused))]
impl Notifier {
    pub async fn new() -> Self {
        let log = RwLock::new(HashMap::new());
        Notifier { log }
    }
}

impl INotifier for Notifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError> {
        let data = serde_json::to_string(message)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?;
        let mut hash_map = self.log.write().unwrap();
        match hash_map.get_mut(connection_id) {
            Some(log) => log.push(data),
            None => {
                hash_map.insert(connection_id.to_string(), vec![data]);
            }
        }
        Ok(())
    }
    fn get_messages(&self, connection_id: &str) -> Vec<String> {
        let hash_map = self.log.read().unwrap();
        hash_map.get(connection_id).unwrap_or(&vec![]).clone()
    }
}
