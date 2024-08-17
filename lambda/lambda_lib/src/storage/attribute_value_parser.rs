use crate::domain::errors::LogicError;
use aws_sdk_dynamodb::types::AttributeValue;

pub trait AttributeValueParser: Sized {
    fn parse(value: Option<&AttributeValue>) -> Result<Self, LogicError>;
}

pub fn parse_attribute_value<T: AttributeValueParser>(
    value: Option<&AttributeValue>,
) -> Result<T, LogicError> {
    T::parse(value)
}

impl AttributeValueParser for String {
    fn parse(value: Option<&AttributeValue>) -> Result<Self, LogicError> {
        let value = value.ok_or(LogicError::DeserializationError(
            "Key not found".to_string(),
        ))?;
        let result = value
            .as_s()
            .map_err(|_| LogicError::DeserializationError("Expected string".to_string()))?
            .clone();
        Ok(result)
    }
}

impl AttributeValueParser for Option<String> {
    fn parse(value: Option<&AttributeValue>) -> Result<Self, LogicError> {
        match value {
            None => Ok(None),
            Some(attr_value) => {
                let result = attr_value
                    .as_s()
                    .map_err(|_| LogicError::DeserializationError("Expected string".to_string()))?
                    .clone();
                Ok(Some(result))
            }
        }
    }
}
impl AttributeValueParser for i32 {
    fn parse(value: Option<&AttributeValue>) -> Result<Self, LogicError> {
        let value = value.ok_or(LogicError::DeserializationError(
            "Key not found".to_string(),
        ))?;
        let result = value
            .as_n()
            .map_err(|_| LogicError::DeserializationError("Expected number".to_string()))?
            .parse::<i32>()
            .map_err(|_| LogicError::DeserializationError("Could not parse number".to_string()))?;
        Ok(result)
    }
}
