use crate::{EventMessage, IEventPublisher};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_eventbridge::{config::Region, types::PutEventsRequestEntry, Client};
use domain::errors::LogicError;
use std::env;

pub struct EventPublisher {
    client: Client,
}

#[cfg_attr(feature = "in_memory", allow(unused))]
impl EventPublisher {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&config);
        EventPublisher { client }
    }
}

impl IEventPublisher for EventPublisher {
    async fn publish(&self, message: &EventMessage) -> Result<(), LogicError> {
        let detail = serde_json::to_string(&message.detail)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?;

        let entry = PutEventsRequestEntry::builder()
            .detail_type(&message.detail_type)
            .detail(&detail)
            .source(&message.source)
            .build();

        self.client
            .put_events()
            .entries(entry)
            .send()
            .await
            .map_err(|e| LogicError::EventPublishingError(e.to_string()))?;
        Ok(())
    }

    fn get_messages(&self, _: &str) -> Vec<EventMessage> {
        vec![]
    }
}
