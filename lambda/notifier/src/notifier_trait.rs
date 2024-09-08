use crate::Message;
use domain::errors::LogicError;

#[trait_variant::make(HttpService: Send)]
pub trait INotifier {
    async fn notify(&self, connection_id: &str, message: &Message) -> Result<(), LogicError>;
    fn get_messages(&self, connection_id: &str) -> Vec<String>;
}
