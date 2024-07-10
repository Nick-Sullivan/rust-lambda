use crate::domain::errors::DatabaseError;

#[derive(Clone)]
pub struct NameCount {
    pub name: String,
    pub count: i32,
}

pub trait INameDatabase {
    async fn get(&self, name: &str) -> Result<NameCount, DatabaseError>;
    async fn save(&mut self, item: &NameCount) -> Result<(), DatabaseError>;
    async fn clear(&mut self, name: &str) -> Result<(), DatabaseError>;
}
