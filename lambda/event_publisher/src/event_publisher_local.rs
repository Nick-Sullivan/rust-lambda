use crate::{EventMessage, IEventPublisher};
use domain::errors::LogicError;
use std::{collections::HashMap, sync::RwLock};

pub struct EventPublisher {
    // pub log: RwLock<Vec<EventMessage>>,
    pub log: RwLock<HashMap<String, Vec<EventMessage>>>,
}

#[cfg_attr(not(feature = "in_memory"), allow(unused))]
impl EventPublisher {
    pub async fn new() -> Self {
        // let log = RwLock::new(Vec::new());
        let log = RwLock::new(HashMap::new());
        EventPublisher { log }
    }
}

impl IEventPublisher for EventPublisher {
    async fn publish(&self, message: &EventMessage) -> Result<(), LogicError> {
        let mut hash_map = self.log.write().unwrap();
        match hash_map.get_mut(&message.source.clone()) {
            Some(log) => log.push(message.clone()),
            None => {
                hash_map.insert(message.source.clone(), vec![message.clone()]);
            }
        }

        Ok(())
    }
    fn get_messages(&self, source: &str) -> Vec<EventMessage> {
        let hash_map = self.log.read().unwrap();
        hash_map.get(source).unwrap_or(&vec![]).clone()
    }
}
