use crate::domain::errors::DatabaseError;

pub trait INameDatabase {
    async fn increment(&mut self, name: &str) -> Result<(), DatabaseError>;
    async fn get_count(&self, name: &str) -> Result<i32, DatabaseError>;
    async fn clear(&mut self, name: &str) -> Result<(), DatabaseError>;
}
