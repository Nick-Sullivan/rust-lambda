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
    SerializationError(String),
    UpdateItemError(String),
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
            LogicError::SerializationError(ref msg) => write!(f, "[SerializationError] {}", msg),
            LogicError::UpdateItemError(ref msg) => write!(f, "[UpdateItemError] {}", msg),
        }
    }
}
impl Error for LogicError {}
