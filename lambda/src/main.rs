mod api;
mod dependency_injection;
mod domain;
mod service;
mod storage;
use lambda_http::{run, service_fn, tracing, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    // run(service_fn(api::api::invoke)).await
    run(service_fn(api::websocket::invoke)).await
}
