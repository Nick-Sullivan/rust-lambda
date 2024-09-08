use crate::{INotifier, Message};
use domain::errors::LogicError;
use serde_json::json;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Notifier {
    pub log: RwLock<HashMap<String, Vec<String>>>,
}

#[cfg_attr(not(feature = "in_memory"), allow(unused))]
impl Notifier {
    pub async fn new() -> Self {
        let log = RwLock::new(HashMap::new());
        Notifier { log }
    }
}

impl INotifier for Notifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError> {
        let message_value = if message.is_error {
            json!({
                "action": message.action.to_string(),
                "error": message.action.get_value()?,
            })
        } else {
            json!({
                "action": message.action.to_string(),
                "data": message.action.get_value()?,
            })
        };
        let message_string = serde_json::to_string(&message_value)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?;
        let mut hash_map = self.log.write().unwrap();
        match hash_map.get_mut(connection_id) {
            Some(log) => log.push(message_string),
            None => {
                hash_map.insert(connection_id.to_string(), vec![message_string]);
            }
        }
        Ok(())
    }
    fn get_messages(&self, connection_id: &str) -> Vec<String> {
        let hash_map = self.log.read().unwrap();
        hash_map.get(connection_id).unwrap_or(&vec![]).clone()
    }
}
