use crate::domain::errors::LogicError;

#[derive(Clone)]
pub struct SessionItem {
    pub session_id: String,
    pub connection_id: String,
    pub version: i32,
}

impl SessionItem {
    pub fn new(session_id: &str, connection_id: &str) -> Self {
        SessionItem {
            session_id: session_id.to_string(),
            connection_id: connection_id.to_string(),
            version: 0,
        }
    }
}
pub trait ISessionTable {
    async fn get(&self, session_id: &str) -> Result<SessionItem, LogicError>;
    async fn save(&mut self, item: &SessionItem) -> Result<(), LogicError>;
    async fn clear(&mut self, session_id: &str) -> Result<(), LogicError>;
}
