pub fn handler(name: &str) -> Result<String, String> {
    if name == "Nick" {
        return Err("Nick is not allowed".to_string());
    }
    let message = format!("Hello {name}");
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_success() {
        let result = handler("Alice");
        assert_eq!(result, Ok("Hello Alice".to_string()));
    }

    #[test]
    fn test_handler_error() {
        let result = handler("Nick");
        assert_eq!(result, Err("Nick is not allowed".to_string()));
    }
}
