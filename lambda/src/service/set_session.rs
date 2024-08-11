use crate::dependency_injection::get_dynamodb_client;
use crate::domain::commands::SetSessionCommand;
use crate::domain::errors::LogicError;
use crate::storage::dynamodb_client::IDynamoDbClient;
use crate::storage::session_table::SessionItem;

pub async fn handler(command: &SetSessionCommand) -> Result<String, LogicError> {
    let db = get_dynamodb_client().await;

    println!("initialising item");
    let item = SessionItem::new(&command.session_id, &command.connection_id);
    println!("creating transaction");
    let transaction = item.save()?;
    let transactions = vec![transaction];
    println!("saving to db");
    let db_lock = db.lock().await;
    db_lock.write(transactions).await?;
    println!("returning");
    Ok("Success".to_string())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_create() {
//         let session_id = "session_id".to_string();
//         let connection_id = "connection_id".to_string();
//         let request = SetSessionCommand {
//             session_id,
//             connection_id,
//         };
//         let result = handler(&request).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_update() {
//         let db = get_dynamodb_client().await;
//         let session_id = "session_id".to_string();
//         let connection_id = "connection_id".to_string();
//         let item = SessionItem::new(&session_id, &connection_id);
//         let transaction = item.save().unwrap();
//         let db_lock = db.lock().await;
//         let _ = db_lock.write(vec![transaction]).await;
//         drop(db_lock);

//         let request = SetSessionCommand {
//             session_id,
//             connection_id,
//         };
//         let result = handler(&request).await;
//         assert!(result.is_err());
//     }
// }
