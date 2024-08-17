use crate::requests::{SayGoodbyeRequest, SayHelloRequest};
use domain::errors::LogicError;
use service;

pub enum HandlerType {
    Goodbye,
    Hello,
}

impl HandlerType {
    pub fn from_str(s: &str) -> Result<HandlerType, String> {
        match s {
            "/v1/hello" => Ok(HandlerType::Hello),
            "/v1/goodbye" => Ok(HandlerType::Goodbye),
            _ => Err("Invalid handler type".to_owned()),
        }
    }
}

pub async fn route(handler_type: &HandlerType, body: &[u8]) -> Result<String, LogicError> {
    match handler_type {
        HandlerType::Hello => {
            let request = deserialise_body::<SayHelloRequest>(body)?;
            let command = request.to_command();
            service::hello::handler(&command).await
        }
        HandlerType::Goodbye => {
            let request = deserialise_body::<SayGoodbyeRequest>(body)?;
            let command = request.to_command();
            service::goodbye::handler(&command).await
        }
    }
}

fn deserialise_body<T>(body_str: &[u8]) -> Result<T, LogicError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(body_str).map_err(|e| LogicError::DeserializationError(e.to_string()))
}
