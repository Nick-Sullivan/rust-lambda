use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum LogicError {
    ConditionalCheckFailed(String),
    DeleteItemError(String),
    DeserializationError(String),
    GetItemError(String),
    NotAllowed,
    ParseItemError(String),
    RestError(String),
    SerializationError(String),
    UpdateItemError(String),
    WebsocketError(String),
}

impl fmt::Display for LogicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogicError::ConditionalCheckFailed(ref msg) => {
                write!(f, "[ConditionalCheckFailed] {}", msg)
            }
            LogicError::DeleteItemError(ref msg) => write!(f, "[DeleteItemError] {}", msg),
            LogicError::DeserializationError(ref msg) => {
                write!(f, "[DeserializationError] {}", msg)
            }
            LogicError::GetItemError(ref msg) => write!(f, "[GetItemError] {}", msg),
            LogicError::NotAllowed => write!(f, "[NotAllowed]"),
            LogicError::ParseItemError(ref msg) => write!(f, "[ParseError] {}", msg),
            LogicError::RestError(ref msg) => write!(f, "[RestError] {}", msg),
            LogicError::SerializationError(ref msg) => write!(f, "[SerializationError] {}", msg),
            LogicError::UpdateItemError(ref msg) => write!(f, "[UpdateItemError] {}", msg),
            LogicError::WebsocketError(ref msg) => write!(f, "[WebsocketError] {}", msg),
        }
    }
}
impl Error for LogicError {}
