use crate::database::{INameDatabase, NameCount};
use domain::errors::LogicError;
use std::collections::HashMap;

pub struct Database {
    counts: HashMap<String, NameCount>,
}

#[cfg_attr(not(feature = "in_memory"), allow(unused))]
impl Database {
    pub async fn new() -> Self {
        let counts = HashMap::new();
        Database { counts }
    }
}

impl INameDatabase for Database {
    async fn get(&self, name: &str) -> Result<NameCount, LogicError> {
        let item = self.counts.get(name);
        match item {
            Some(item) => Ok(item.clone()),
            None => Ok(NameCount::new(name)),
        }
    }

    async fn save(&mut self, item: &NameCount) -> Result<(), LogicError> {
        self.counts.insert(item.name.clone(), item.clone());
        Ok(())
    }

    async fn clear(&mut self, name: &str) -> Result<(), LogicError> {
        self.counts.remove(name);
        Ok(())
    }
}
