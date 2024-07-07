use crate::domain::errors::DatabaseError;
use crate::storage::database::INameDatabase;
use std::collections::HashMap;

pub struct NameCounter {
    counts: HashMap<String, i32>,
}

#[cfg_attr(not(test), allow(unused))]
impl NameCounter {
    pub async fn new() -> Self {
        let counts = HashMap::new();
        NameCounter { counts }
    }
}

impl INameDatabase for NameCounter {
    async fn increment(&mut self, name: &str) -> Result<(), DatabaseError> {
        let count = self.counts.entry(name.to_string()).or_insert(0);
        *count += 1;
        Ok(())
    }

    async fn get_count(&self, name: &str) -> Result<i32, DatabaseError> {
        let count = *self.counts.get(name).unwrap_or(&0);
        Ok(count)
    }

    async fn clear(&mut self, name: &str) -> Result<(), DatabaseError> {
        self.counts.remove(name);
        Ok(())
    }
}
