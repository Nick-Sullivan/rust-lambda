use crate::{GameStateMessage, SetNicknameMessage};
use domain::errors::LogicError;
use serde;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum ActionType {
    DestroySession(String),
    GameState(GameStateMessage),
    GetSession(String),
    JoinGame(String),
    SetNickname(SetNicknameMessage),
    SetNicknameFailure(String),
}
impl ActionType {
    pub fn to_string(&self) -> String {
        match self {
            ActionType::DestroySession(_) => "destroySession".to_string(),
            ActionType::GameState(_) => "gameState".to_string(),
            ActionType::GetSession(_) => "getSession".to_string(),
            ActionType::JoinGame(_) => "joinGame".to_string(),
            ActionType::SetNickname(_) => "setNickname".to_string(),
            ActionType::SetNicknameFailure(_) => "setNickname".to_string(),
        }
    }

    pub fn get_value(&self) -> Result<String, LogicError> {
        match self {
            ActionType::DestroySession(data) => Ok(data.clone()),
            ActionType::GameState(data) => ActionType::serialize_data(&data),
            ActionType::GetSession(data) => Ok(data.clone()),
            ActionType::JoinGame(data) => Ok(data.clone()),
            ActionType::SetNickname(data) => ActionType::serialize_data(&data),
            ActionType::SetNicknameFailure(data) => Ok(data.clone()),
        }
    }

    fn serialize_data<T: serde::Serialize>(data: &T) -> Result<String, LogicError> {
        serde_json::to_string(data).map_err(|e| LogicError::SerializationError(e.to_string()))
    }
}

// #[derive(Serialize, Debug)]
pub struct Message {
    pub action: ActionType,
    pub is_error: bool,
    // pub data: Option<Value>,
    // pub error: Option<Value>,
}
impl Message {
    pub fn new(action: ActionType) -> Self {
        Self {
            action,
            is_error: false,
        }
    }
    pub fn new_err(action: ActionType) -> Self {
        Self {
            action,
            is_error: true,
        }
    }
}
