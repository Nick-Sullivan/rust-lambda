use crate::dependency_injection::DATABASE;
use crate::domain::commands::SayHelloCommand;
use crate::storage::database::INameDatabase;

pub fn handler(command: SayHelloCommand) -> Result<String, String> {
    if command.name == "Nick" {
        return Err("Nick is not allowed".to_string());
    }
    let mut db = DATABASE.lock().unwrap();
    db.increment(&command.name);
    let count = db.get_count(&command.name);
    let message = format!("Hello {0}, {1} times", command.name, count);
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_success() {
        let request = SayHelloCommand {
            name: "Alice".to_string(),
        };
        let result = handler(request);
        assert_eq!(result, Ok("Hello Alice".to_string()));
    }

    #[test]
    fn test_handler_error() {
        let request = SayHelloCommand {
            name: "Nick".to_string(),
        };
        let result = handler(request);
        assert_eq!(result, Err("Nick is not allowed".to_string()));
    }
}
