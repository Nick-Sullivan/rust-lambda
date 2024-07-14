use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    UpdateItemError(String),
    ParseError(String),
    ConditionalCheckFailed(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DatabaseError::ConnectionError(ref msg) => {
                write!(f, "[ConnectionError] {}", msg)
            }
            DatabaseError::ParseError(ref msg) => write!(f, "[ParseError] {}", msg),
            DatabaseError::UpdateItemError(ref msg) => write!(f, "[UpdateItemError] {}", msg),
            DatabaseError::ConditionalCheckFailed(ref msg) => {
                write!(f, "[ConditionalCheckFailed] {}", msg)
            }
        }
    }
}
impl Error for DatabaseError {}

#[derive(Debug, PartialEq)]
pub enum HandlerError {
    DatabaseError(String),
    DeserializationError(String),
    NotAllowed,
}

impl From<DatabaseError> for HandlerError {
    fn from(error: DatabaseError) -> Self {
        HandlerError::DatabaseError(error.to_string())
    }
}

impl From<String> for HandlerError {
    fn from(err: String) -> HandlerError {
        HandlerError::DeserializationError(err)
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandlerError::NotAllowed => write!(f, "Item not found"),
            HandlerError::DatabaseError(ref cause) => write!(f, "Database error: {}", cause),
            HandlerError::DeserializationError(ref cause) => {
                write!(f, "Deserialisation error: {}", cause)
            }
        }
    }
}
impl Error for HandlerError {}
