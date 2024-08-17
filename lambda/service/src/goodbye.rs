use domain::commands::SayGoodbyeCommand;
use domain::errors::LogicError;
use storage::database::INameDatabase;
use storage::dependency_injection::get_database;

pub async fn handler(command: &SayGoodbyeCommand) -> Result<String, LogicError> {
    if command.name == "Nick" {
        return Err(LogicError::NotAllowed);
    }
    let db = get_database().await;
    let mut db_lock = db.lock().await;
    let item = db_lock.get(&command.name).await?;
    let message = format!("Goodbye {0}, {1} times", command.name, item.count);
    db_lock.clear(&command.name).await?;
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::drop;
    use storage::database::NameCount;

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
        let item = NameCount {
            name: name.to_string(),
            count: 1,
            version: 0,
        };
        let db = get_database().await;
        let mut db_lock = db.lock().await;
        let _ = db_lock.save(&item).await;
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
        assert_eq!(result.unwrap_err(), LogicError::NotAllowed);
    }
}
