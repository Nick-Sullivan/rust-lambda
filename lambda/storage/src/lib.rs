pub mod attribute_value_parser;
mod database_cloud;
mod database_instance;
mod database_local;
mod database_trait;
mod dynamodb_client_cloud;
mod dynamodb_client_instance;
mod dynamodb_client_local;
mod dynamodb_client_trait;
pub mod game_table;
pub mod session_table;
pub mod websocket_table;

#[cfg(not(feature = "in_memory"))]
pub use database_cloud::Database;
pub use database_instance::get_database;
#[cfg(feature = "in_memory")]
pub use database_local::Database;
pub use database_trait::{INameDatabase, NameCount};

#[cfg(not(feature = "in_memory"))]
pub use dynamodb_client_cloud::DynamoDbClient;
pub use dynamodb_client_instance::get;
#[cfg(feature = "in_memory")]
pub use dynamodb_client_local::DynamoDbClient;
pub use dynamodb_client_trait::IDynamoDbClient;
