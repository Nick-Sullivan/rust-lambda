use lambda_http::aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use lambda_http::{Body, Error, Response};
use lambda_lib::api::websocket;
use lambda_lib::domain::errors::LogicError;

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

    let request_type = websocket::get_request_type(&route_key, &body_str)?;

    let result = websocket::route(&request_type, &connection_id).await;
    match result {
        Ok(message) => Ok(Response::new(Body::Text(message))),
        Err(e) => {
            let message = format!("Error: {e}");
            Err(Box::new(LogicError::WebsocketError(message)))
        }
    }
}
