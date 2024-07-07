use crate::dependency_injection::get_database;
use crate::domain::commands::SayGoodbyeCommand;
use crate::domain::errors::HandlerError;
use crate::storage::database::INameDatabase;

pub async fn handler(command: &SayGoodbyeCommand) -> Result<String, HandlerError> {
    if command.name == "Nick" {
        return Err(HandlerError::NotAllowed);
    }
    let db = get_database().await;
    let mut db_lock = db.lock().await;
    let count = db_lock.get_count(&command.name).await?;
    db_lock.clear(&command.name).await?;
    let message = format!("Goodbye {0}, {1} times", command.name, count);
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::drop;

    #[tokio::test]
    async fn test_initial_goodbye() {
        let name = "test_initial_goodbye".to_string();
        let request = SayGoodbyeCommand { name };
        let result = handler(&request).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Goodbye test_initial_goodbye, 0 times".to_string()
        );
    }

    #[tokio::test]
    async fn test_second_goodbye() {
        let name = "test_second_goodbye".to_string();
        let db = get_database().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.increment(&name).await;
        drop(db_lock);

        let request = SayGoodbyeCommand { name };
        let result = handler(&request).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Goodbye test_second_goodbye, 1 times".to_string()
        );
    }

    #[tokio::test]
    async fn test_invalid_goodbye() {
        let name = "Nick".to_string();
        let request = SayGoodbyeCommand { name };
        let result = handler(&request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), HandlerError::NotAllowed);
    }
}
