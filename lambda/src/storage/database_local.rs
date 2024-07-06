use crate::storage::database::INameDatabase;
use std::collections::HashMap;

pub struct NameCounter {
    counts: HashMap<String, i32>,
}

#[cfg_attr(not(feature = "is_local"), allow(unused))]
impl NameCounter {
    pub fn new() -> Self {
        NameCounter {
            counts: HashMap::new(),
        }
    }
}

impl INameDatabase for NameCounter {
    fn increment(&mut self, name: &str) {
        let count = self.counts.entry(name.to_string()).or_insert(0);
        *count += 1;
    }

    fn get_count(&self, name: &str) -> i32 {
        *self.counts.get(name).unwrap_or(&0)
    }

    fn clear(&mut self, name: &str) {
        self.counts.remove(name);
    }
}
