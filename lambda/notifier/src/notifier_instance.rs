use crate::Notifier;
use std::sync::Arc;
use tokio::sync::OnceCell;

static NOTIFIER: OnceCell<Arc<Notifier>> = OnceCell::const_new();

pub async fn get() -> Arc<Notifier> {
    NOTIFIER.get_or_init(init).await.clone()
}

async fn init() -> Arc<Notifier> {
    let notifier = Notifier::new().await;
    Arc::new(notifier)
}
