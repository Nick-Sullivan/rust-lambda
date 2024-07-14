use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_apigatewaymanagement::{config::Region, primitives::Blob, Client};
use lambda_http::{Body, Error, Request, RequestExt, Response};
use std::env;

#[derive(serde::Deserialize)]
struct Context {
    #[serde(rename = "routeKey")]
    route_key: String,
    #[serde(rename = "connectionId")]
    connection_id: String,
}

pub async fn invoke(event: Request) -> Result<Response<(Body)>, Error> {
    let method = event.method();
    println!("Method: {method}");
    let path = event.uri().path();
    println!("Path: {path}");
    let ctx = event.request_context().clone();
    let ctx_str = serde_json::to_string(&ctx)?;
    println!("ctx_str: {ctx_str}");
    let context: Context = serde_json::from_str(&ctx_str)?;
    let route_key = context.route_key.clone();
    println!("route_key: {route_key}");
    let body = event.body();
    let body_str = std::str::from_utf8(body)?;
    println!("Body: {body_str}");
    let connection_id = context.connection_id.clone();
    println!("connection_id: {connection_id}");

    if route_key == "$connect" || route_key == "$disconnect" {
        return Ok(Response::new(Body::Empty));
    }

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
        .connection_id(context.connection_id.clone())
        .data(Blob::new(message.as_bytes().to_vec()))
        .send()
        .await?;

    Ok(Response::new(Body::Empty))
}
