use crate::dependency_injection::get_notifier;
use crate::notifier::notifier::INotifier;
use crate::{domain::errors::LogicError, service};
use lambda_http::{
    aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext, Body, Error, Response,
};

use super::requests;

enum RequestType {
    Connect(requests::CreateConnectionRequest),
    Disconnect(requests::DestroyConnectionRequest),
    SetSession(requests::SetSessionRequest),
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
        Body::Binary(_) => Err(LogicError::WebsocketError(
            "Binary not supported".to_string(),
        )),
        Body::Empty => Ok("".to_string()),
        Body::Text(s) => Ok(s.to_string()),
    }?;

    println!("Body: {body_str}");

    let request_type = get_request_type(&route_key, &body_str)?;

    let result = route(&request_type, &connection_id).await;
    match result {
        Ok(message) => Ok(Response::new(Body::Text(message))),
        Err(e) => {
            let message = format!("Error: {e}");
            Err(Box::new(LogicError::WebsocketError(message)))
        }
    }
}

fn get_request_type(route_key: &str, body_str: &str) -> Result<RequestType, LogicError> {
    if route_key == "$connect" {
        return Ok(RequestType::Connect(requests::CreateConnectionRequest {}));
    }
    if route_key == "$disconnect" {
        return Ok(RequestType::Disconnect(
            requests::DestroyConnectionRequest {},
        ));
    }

    let request: requests::WebsocketRequest = serde_json::from_str(&body_str)
        .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
    println!("Request action {}", request.action);
    println!("Request data {}", request.data);
    match request.action.as_str() {
        "setSession" => {
            let request: requests::SetSessionRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::SetSession(request))
        }
        _ => Err(LogicError::WebsocketError("Unknown action".to_string()))?,
    }
}
async fn route(request_type: &RequestType, connection_id: &str) -> Result<String, LogicError> {
    match request_type {
        RequestType::Connect(request) => {
            let command = request.to_command(connection_id);
            service::create_connection::handler(&command).await
        }
        RequestType::Disconnect(request) => {
            let command = request.to_command(connection_id);
            service::destroy_connection::handler(&command).await
        }
        RequestType::SetSession(request) => {
            let command = request.to_command(connection_id);
            service::set_session::handler(&command).await
        } // HandlerType::Connect => {
          //     let request: SayHelloRequest = serde_json::from_str(body)
          //         .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
          //     let command = request.to_command();
          //     service::hello::handler(&command).await
          // }
          // HandlerType::Disconnect => {
          //     let request: SayGoodbyeRequest = serde_json::from_str(body)
          //         .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
          //     let command = request.to_command();
          //     service::goodbye::handler(&command).await
          // }
          // RequestType::Default => handler(body, connection_id).await,
    }
}

// pub async fn handler(body_str: &str, connection_id: &str) -> Result<String, LogicError> {
//     let notifier = get_notifier().await;
//     let message = format!("Responding to {body_str}");
//     notifier.notify(&connection_id, &message).await?;
//     Ok(message.to_string())
// }
