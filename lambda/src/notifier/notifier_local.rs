use crate::domain::errors::LogicError;
use crate::notifier::notifier::INotifier;

pub struct Notifier {}

#[cfg_attr(not(test), allow(unused))]
impl Notifier {
    pub async fn new() -> Self {
        Notifier {}
    }
}

impl INotifier for Notifier {
    async fn notify(&self, connection_id: &str, message: &str) -> Result<(), LogicError> {
        Ok(())
    }
}
