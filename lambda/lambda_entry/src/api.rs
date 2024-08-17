use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequestContext;
use lambda_http::{Body, Error, Response};
use lambda_lib::api::api;
use lambda_lib::domain::errors::LogicError;

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

    let handler_type = api::HandlerType::from_str(&path)?;
    let body_str = match body {
        Body::Empty => Ok("".to_string()),
        Body::Text(s) => Ok(s.to_string()),
        Body::Binary(_) => Err(LogicError::WebsocketError(
            "Binary not supported".to_string(),
        )),
    }?;
    let max_retries = 10;

    for _ in 0..max_retries {
        let result = api::route(&handler_type, body_str.as_bytes()).await;
        if let Err(LogicError::ConditionalCheckFailed(_)) = result {
            continue;
        }
        return build_response(result);
    }

    build_response(Err(LogicError::ConditionalCheckFailed(
        "Max retries reached".into(),
    )))
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
