use crate::domain::commands;
use serde::{Deserialize, Serialize};

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
