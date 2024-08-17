use super::notifier::Message;
use crate::domain::errors::LogicError;
use crate::notifier::notifier::INotifier;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_apigatewaymanagement::{config::Region, primitives::Blob, Client};
use std::env;

pub struct Notifier {
    client: Client,
}

#[cfg_attr(test, allow(unused))]
impl Notifier {
    pub async fn new() -> Self {
        let region_name = env::var("AWS_REGION").unwrap_or_else(|_| "".to_string());
        let gateway_url = env::var("API_GATEWAY_URL").unwrap_or_else(|_| "".to_string());
        let region_provider =
            RegionProviderChain::first_try(Region::new(region_name)).or_default_provider();
        let config = aws_config::from_env()
            .region(region_provider)
            .endpoint_url(gateway_url.replace("wss", "https"))
            .load()
            .await;
        let client = Client::new(&config);
        Notifier { client }
    }
}

impl INotifier for Notifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError> {
        let data = serde_json::to_string(message)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?;
        self.client
            .post_to_connection()
            .connection_id(connection_id)
            .data(Blob::new(data.as_bytes().to_vec()))
            .send()
            .await
            .map_err(|e| LogicError::WebsocketError(e.to_string()))?;
        Ok(())
    }
}
