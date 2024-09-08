use crate::EventPublisher;
use std::sync::Arc;
use tokio::sync::OnceCell;

static PUBLISHER: OnceCell<Arc<EventPublisher>> = OnceCell::const_new();

pub async fn get() -> Arc<EventPublisher> {
    PUBLISHER.get_or_init(init).await.clone()
}

async fn init() -> Arc<EventPublisher> {
    let event_publisher = EventPublisher::new().await;
    Arc::new(event_publisher)
}
