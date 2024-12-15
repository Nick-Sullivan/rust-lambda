use aws_lambda_events::event::sqs::SqsEvent;
use domain::{commands::CheckSessionTimeoutCommand, errors::LogicError};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(event: LambdaEvent<SqsEvent>) -> Result<Value, Error> {
    let sqs_event = event.payload;
    let records = sqs_event.records;
    println!("Received records: {:?}", records);

    for record in records {
        let body_str = record
            .body
            .ok_or(LogicError::LambdaError("body is required".to_string()))?;
        let body = serde_json::from_str::<Value>(&body_str)
            .map_err(|e| LogicError::LambdaError(format!("Failed to parse body: {}", e)))?;
        let session_id = body["detail"]["session_id"]
            .as_str()
            .ok_or(LogicError::LambdaError(
                "session_id is required".to_string(),
            ))?;
        println!("Received session_id: {:?}", session_id);
        let command = CheckSessionTimeoutCommand {
            session_id: session_id.to_string(),
        };
        let message = service::check_session_timeout::handler(&command).await?;
        println!("Message: {:?}", message);
    }
    Ok(json!({"hello": "world"}))
}
