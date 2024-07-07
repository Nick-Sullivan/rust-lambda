use crate::dependency_injection::get_database;
use crate::domain::commands::SayHelloCommand;
use crate::domain::errors::HandlerError;
use crate::storage::database::INameDatabase;

pub async fn handler(command: &SayHelloCommand) -> Result<String, HandlerError> {
    if command.name == "Nick" {
        return Err(HandlerError::NotAllowed);
    }
    let db = get_database().await;
    let mut db_lock = db.lock().await;
    db_lock.increment(&command.name).await?;
    let count = db_lock.get_count(&command.name).await?;
    let message = format!("Hello {0}, {1} times", command.name, count);
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_hello() {
        let name = "test_initial_hello".to_string();
        let request = SayHelloCommand { name };
        let result = handler(&request).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello test_initial_hello, 1 times".to_string()
        );
    }

    #[tokio::test]
    async fn test_second_hello() {
        let name = "test_second_hello".to_string();
        let db = get_database().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.increment(&name).await;
        drop(db_lock);

        let request = SayHelloCommand { name };
        let result = handler(&request).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello test_second_hello, 2 times".to_string()
        );
    }

    #[tokio::test]
    async fn test_invalid_hello() {
        let name = "Nick".to_string();
        let request = SayHelloCommand { name };
        let result = handler(&request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), HandlerError::NotAllowed);
    }
}
