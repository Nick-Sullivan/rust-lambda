use domain::errors::LogicError;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
pub enum ActionType {
    GameState,
    GetSession,
    JoinGame,
    SetNickname,
}

#[derive(Serialize, Debug)]
pub struct Message {
    pub action: ActionType,
    pub data: Option<Value>,
    pub error: Option<Value>,
}
impl Message {
    pub fn new(action: ActionType, data: Value) -> Self {
        Self {
            action,
            data: Some(data),
            error: None,
        }
    }
    pub fn new_err(action: ActionType, error: Value) -> Self {
        Self {
            action,
            data: None,
            error: Some(error),
        }
    }
}

#[trait_variant::make(HttpService: Send)]
pub trait INotifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError>;
    fn get_messages(&self, connection_id: &str) -> Vec<String>;
}
