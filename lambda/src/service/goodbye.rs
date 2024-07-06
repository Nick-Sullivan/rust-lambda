use crate::dependency_injection::DATABASE;
use crate::domain::commands::SayGoodbyeCommand;
use crate::storage::database::INameDatabase;

pub fn handler(command: SayGoodbyeCommand) -> Result<String, String> {
    if command.name == "Nick" {
        return Err("Nick is not allowed".to_string());
    }
    let mut db = DATABASE.lock().unwrap();
    let count = db.get_count(&command.name);
    db.clear(&command.name);
    let message = format!("Goodbye {0}, {1} times", command.name, count);
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_success() {
        let request = SayGoodbyeCommand {
            name: "Alice".to_string(),
        };
        let result = handler(request);
        assert_eq!(result, Ok("Goodbye Alice".to_string()));
    }

    #[test]
    fn test_handler_error() {
        let request = SayGoodbyeCommand {
            name: "Nick".to_string(),
        };
        let result = handler(request);
        assert_eq!(result, Err("Nick is not allowed".to_string()));
    }
}
