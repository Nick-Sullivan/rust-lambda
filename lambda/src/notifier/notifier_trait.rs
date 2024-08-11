use crate::domain::errors::LogicError;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum ActionType {
    GetSession,
}

#[derive(Serialize, Debug)]
pub struct Message {
    pub action: ActionType,
    pub data: String,
}

pub trait INotifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError>;
}
