use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SayHelloCommand {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SayGoodbyeCommand {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateConnectionCommand {
    pub connection_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGameCommand {
    pub connection_id: String,
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionCommand {
    pub connection_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DestroyConnectionCommand {
    pub connection_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetNicknameCommand {
    pub connection_id: String,
    pub session_id: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetSessionCommand {
    pub connection_id: String,
    pub session_id: String,
}
