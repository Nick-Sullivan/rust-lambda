mod api;
use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use tracing::info;

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

#[derive(Serialize, Deserialize, Debug)]
struct MyRequestBody {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(invoke)).await
}

async fn invoke(event: Request) -> Result<Response<Body>, Error> {
    let _method = event.method().as_str();
    let path = event.uri().path();
    let body_str = event.body().as_ref();
    info!("Path: {path}");

    let handler_type = HandlerType::from_str(&path)?;
    let result = route(handler_type, body_str);

    match result {
        Ok(message) => {
            let resp = Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(message.into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
        Err(e) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(e.into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
    }
}

fn route(handler_type: HandlerType, body_str: &[u8]) -> Result<String, String> {
    match handler_type {
        HandlerType::Hello => {
            let body = deserialise_body::<MyRequestBody>(body_str)?;
            api::hello::handler(&body.name)
        }
        HandlerType::Goodbye => {
            let body = deserialise_body::<MyRequestBody>(body_str)?;
            api::goodbye::handler(&body.name)
        }
    }
}

fn deserialise_body<T>(body_str: &[u8]) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(body_str).map_err(|e| e.to_string())
}

// fn lookup_handler(handler_type: HandlerType) -> fn(&str) -> Result<String, String> {
//     match handler_type {
//         HandlerType::Hello => api::hello::handler,
//         HandlerType::Goodbye => api::goodbye::handler,
//         HandlerType::Local => api::hello::handler,
//     }
// }
