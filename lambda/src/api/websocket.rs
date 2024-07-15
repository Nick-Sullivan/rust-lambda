use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_apigatewaymanagement::{config::Region, primitives::Blob, Client};
use lambda_http::{
    aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext, Body, Error, Response,
};
use std::env;

use crate::domain::errors::LogicError;

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

    if route_key == "$connect" || route_key == "$disconnect" {
        return Ok(Response::new(Body::Empty));
    }

    let body_str = match body {
        Body::Empty => Ok("".to_string()),
        Body::Text(s) => Ok(s.to_string()),
        Body::Binary(_) => Err(LogicError::WebsocketError(
            "Binary not supported".to_string(),
        )),
    }?;
    println!("Body: {body_str}");

    // Send response
    let region_name = env::var("AWS_REGION")?;
    let gateway_url = env::var("API_GATEWAY_URL")?;
    println!("gateway_url: {gateway_url}");
    let region_provider =
        RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
    let config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url(gateway_url.replace("wss", "https"))
        .load()
        .await;
    let client = Client::new(&config);

    let message = format!("Responding to {body_str}");

    let _response = client
        .post_to_connection()
        .connection_id(connection_id.clone())
        .data(Blob::new(message.as_bytes().to_vec()))
        .send()
        .await?;

    Ok(Response::new(Body::Empty))
}
