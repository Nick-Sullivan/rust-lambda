use lambda_http::request::RequestContext;
use lambda_http::{self, Body, Error, Request, RequestExt, Response};
mod api;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::tracing::init_default_subscriber();
    lambda_http::run(lambda_http::service_fn(invoke)).await
}

pub async fn invoke(event: Request) -> Result<Response<Body>, Error> {
    let ctx = event.request_context();
    let ctx_str = serde_json::to_string(&ctx)?;
    println!("ctx_str: {ctx_str}");
    let body = event.body();
    match ctx {
        RequestContext::ApiGatewayV1(ctx) => api::invoke(&body, &ctx).await,
        RequestContext::WebSocket(ctx) => websocket::invoke(&body, &ctx).await,
        _ => Err(Error::from("Invalid request context")),
    }
}
