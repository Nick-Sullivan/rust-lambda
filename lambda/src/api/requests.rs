use crate::domain::commands;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct SayHelloRequest {
    pub name: String,
}
impl SayHelloRequest {
    pub fn to_command(&self) -> commands::SayHelloCommand {
        commands::SayHelloCommand {
            name: self.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SayGoodbyeRequest {
    pub name: String,
}
impl SayGoodbyeRequest {
    pub fn to_command(&self) -> commands::SayGoodbyeCommand {
        commands::SayGoodbyeCommand {
            name: self.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateConnectionRequest {}
impl CreateConnectionRequest {
    pub fn to_command(&self, connection_id: &str) -> commands::CreateConnectionCommand {
        commands::CreateConnectionCommand {
            connection_id: connection_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionRequest {}
impl CreateSessionRequest {
    pub fn to_command(&self, connection_id: &str) -> commands::CreateSessionCommand {
        commands::CreateSessionCommand {
            connection_id: connection_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DestroyConnectionRequest {}
impl DestroyConnectionRequest {
    pub fn to_command(&self, connection_id: &str) -> commands::DestroyConnectionCommand {
        commands::DestroyConnectionCommand {
            connection_id: connection_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetSessionRequest {
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
impl SetSessionRequest {
    pub fn to_command(&self, connection_id: &str) -> commands::SetSessionCommand {
        commands::SetSessionCommand {
            connection_id: connection_id.to_string(),
            session_id: self.session_id.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketRequest {
    pub action: String,
    pub data: Value,
}
