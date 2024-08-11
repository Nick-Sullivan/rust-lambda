mod api;
mod dependency_injection;
mod domain;
mod notifier;
mod service;
mod storage;
#[cfg(test)]
mod test_setup;
use lambda_http::request::RequestContext;
use lambda_http::{run, service_fn, tracing, Error};
use lambda_http::{Body, Request, RequestExt, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(invoke)).await
}

pub async fn invoke(event: Request) -> Result<Response<Body>, Error> {
    let ctx = event.request_context();
    let ctx_str = serde_json::to_string(&ctx)?;
    println!("ctx_str: {ctx_str}");
    let body = event.body();
    match ctx {
        RequestContext::ApiGatewayV1(ctx) => api::api::invoke(&body, &ctx).await,
        RequestContext::WebSocket(ctx) => api::websocket::invoke(&body, &ctx).await,
        _ => Err(Error::from("Invalid request context")),
    }
}
