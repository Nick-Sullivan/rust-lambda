use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SayHelloCommand {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SayGoodbyeCommand {
    pub name: String,
}
