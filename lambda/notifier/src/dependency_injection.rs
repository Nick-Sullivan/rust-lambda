use crate::notifier::Notifier;
use std::sync::Arc;
use tokio::sync::OnceCell;

static NOTIFIER: OnceCell<Arc<Notifier>> = OnceCell::const_new();

pub async fn get_notifier() -> Arc<Notifier> {
    NOTIFIER.get_or_init(init_notifier).await.clone()
}

async fn init_notifier() -> Arc<Notifier> {
    let notifier = Notifier::new().await;
    Arc::new(notifier)
}
