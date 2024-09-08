use domain::errors::LogicError;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct EventMessage {
    pub source: String,
    pub detail_type: String,
    pub detail: Value,
}

#[trait_variant::make(HttpService: Send)]
pub trait IEventPublisher {
    async fn publish(&self, message: &EventMessage) -> Result<(), LogicError>;
    fn get_messages(&self, source: &str) -> Vec<EventMessage>;
}
