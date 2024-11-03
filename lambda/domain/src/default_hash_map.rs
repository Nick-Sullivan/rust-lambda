use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct DefaultHashMap<K, V> {
    map: HashMap<K, V>,
    default: V,
}

impl<K, V> DefaultHashMap<K, V>
where
    K: Eq + Hash,
    V: Default + Clone,
{
    pub fn new(default: V) -> Self {
        DefaultHashMap {
            map: HashMap::new(),
            default,
        }
    }

    pub fn get(&self, key: &K) -> &V {
        self.map.get(key).unwrap_or(&self.default)
    }
}

impl<K, V> Deref for DefaultHashMap<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<K, V> DerefMut for DefaultHashMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
