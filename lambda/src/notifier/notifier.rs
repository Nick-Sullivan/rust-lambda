use crate::domain::errors::LogicError;

pub trait INotifier {
    async fn notify(&self, connection_id: &str, message: &str) -> Result<(), LogicError>;
}
