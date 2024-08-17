use domain::errors::LogicError;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
pub enum ActionType {
    GetSession,
    SetNickname,
}

#[derive(Serialize, Debug)]
pub struct Message {
    pub action: ActionType,
    pub data: Option<Value>,
    pub error: Option<Value>,
}

pub trait INotifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError>;
    fn get_messages(&self, connection_id: &str) -> Vec<String>;
}
