use crate::api::requests::{SayGoodbyeRequest, SayHelloRequest};
use crate::service;
use lambda_http::{tracing, Body, Error, Request, Response};
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

pub async fn invoke(event: Request) -> Result<Response<Body>, Error> {
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

fn route(handler_type: HandlerType, body: &[u8]) -> Result<String, String> {
    match handler_type {
        HandlerType::Hello => {
            let request = deserialise_body::<SayHelloRequest>(body)?;
            let command = request.to_command();
            service::hello::handler(command)
        }
        HandlerType::Goodbye => {
            let request = deserialise_body::<SayGoodbyeRequest>(body)?;
            let command = request.to_command();
            service::goodbye::handler(command)
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
