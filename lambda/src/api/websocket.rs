use crate::api::requests::{SayGoodbyeRequest, SayHelloRequest};
use crate::dependency_injection::get_notifier;
use crate::notifier::notifier::INotifier;
use crate::{domain::errors::LogicError, service};
use lambda_http::{
    aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext, Body, Error, Response,
};

enum HandlerType {
    Connect,
    Disconnect,
    Default,
}

impl HandlerType {
    fn from_str(s: &str) -> Result<HandlerType, String> {
        match s {
            "$connect" => Ok(HandlerType::Connect),
            "$disconnect" => Ok(HandlerType::Disconnect),
            "$default" => Ok(HandlerType::Default),
            _ => Err("Invalid handler type".to_owned()),
        }
    }
}

pub async fn invoke(
    body: &Body,
    context: &ApiGatewayWebsocketProxyRequestContext,
) -> Result<Response<Body>, Error> {
    let route_key = context
        .route_key
        .clone()
        .ok_or(LogicError::WebsocketError("No route key".to_string()))?;
    let connection_id = context
        .connection_id
        .clone()
        .ok_or(LogicError::WebsocketError("No connection ID".to_string()))?;
    println!("route_key: {route_key}");
    println!("connection_id: {connection_id}");

    let body_str = match body {
        Body::Empty => Ok("".to_string()),
        Body::Text(s) => Ok(s.to_string()),
        Body::Binary(_) => Err(LogicError::WebsocketError(
            "Binary not supported".to_string(),
        )),
    }?;
    println!("Body: {body_str}");
    let handler_type = HandlerType::from_str(&route_key)?;

    let result = route(&handler_type, &body_str, &connection_id).await;

    Ok(Response::new(Body::Empty))
}

async fn route(
    handler_type: &HandlerType,
    body: &str,
    connection_id: &str,
) -> Result<String, LogicError> {
    match handler_type {
        HandlerType::Connect => {
            let request: SayHelloRequest = serde_json::from_str(body)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            let command = request.to_command();
            service::hello::handler(&command).await
        }
        HandlerType::Disconnect => {
            let request: SayGoodbyeRequest = serde_json::from_str(body)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            let command = request.to_command();
            service::goodbye::handler(&command).await
        }
        HandlerType::Default => handler(body, connection_id).await,
    }
}

pub async fn handler(body_str: &str, connection_id: &str) -> Result<String, LogicError> {
    let notifier = get_notifier().await;
    let message = format!("Responding to {body_str}");
    notifier.notify(&connection_id, &message).await?;
    Ok(message.to_string())
}
