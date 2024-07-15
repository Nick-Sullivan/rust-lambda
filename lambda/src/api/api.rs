use crate::api::requests::{SayGoodbyeRequest, SayHelloRequest};
use crate::domain::errors::LogicError;
use crate::service;
use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequestContext;
use lambda_http::{Body, Error, Response};

enum HandlerType {
    Goodbye,
    Hello,
}

impl HandlerType {
    fn from_str(s: &str) -> Result<HandlerType, String> {
        match s {
            "/v1/hello" => Ok(HandlerType::Hello),
            "/v1/goodbye" => Ok(HandlerType::Goodbye),
            _ => Err("Invalid handler type".to_owned()),
        }
    }
}

pub async fn invoke(
    body: &Body,
    context: &ApiGatewayProxyRequestContext,
) -> Result<Response<Body>, Error> {
    let path = context
        .path
        .clone()
        .ok_or(LogicError::RestError("No path".to_string()))?;
    let email = context.authorizer.fields["claims"]["email"]
        .as_str()
        .ok_or(LogicError::RestError("No email".to_string()))?;
    let username = context.authorizer.fields["claims"]["cognito:username"]
        .as_str()
        .ok_or(LogicError::RestError("No username".to_string()))?;
    println!("Path: {path}");
    println!("Email: {email}");
    println!("Username: {username}");

    let handler_type = HandlerType::from_str(&path)?;
    let body_str = match body {
        Body::Empty => Ok("".to_string()),
        Body::Text(s) => Ok(s.to_string()),
        Body::Binary(_) => Err(LogicError::WebsocketError(
            "Binary not supported".to_string(),
        )),
    }?;
    let max_retries = 10;

    for _ in 0..max_retries {
        let result = route(&handler_type, body_str.as_bytes()).await;
        if let Err(LogicError::ConditionalCheckFailed(_)) = result {
            continue;
        }
        return build_response(result);
    }

    build_response(Err(LogicError::ConditionalCheckFailed(
        "Max retries reached".into(),
    )))
}

async fn route(handler_type: &HandlerType, body: &[u8]) -> Result<String, LogicError> {
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

fn build_response(result: Result<String, LogicError>) -> Result<Response<Body>, Error> {
    match result {
        Ok(message) => {
            let resp = Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(message.into())
                .map_err(|e| LogicError::SerializationError(e.to_string()))?;
            Ok(resp)
        }
        Err(e) => {
            let error_message = format!(r#"{{"error": "{}"}}"#, e);
            let resp = Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(error_message.into())
                .map_err(|e| LogicError::SerializationError(e.to_string()))?;
            Ok(resp)
        }
    }
}
