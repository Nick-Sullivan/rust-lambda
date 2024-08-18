pub mod attribute_value_parser;
mod database_cloud;
mod database_local;
mod database_trait;
pub mod dependency_injection;
mod dynamodb_client_cloud;
mod dynamodb_client_local;
mod dynamodb_client_trait;
pub mod game_table;
pub mod session_table;
pub mod websocket_table;
pub mod database {
    #[cfg(not(feature = "in_memory"))]
    pub use super::database_cloud::*;
    #[cfg(feature = "in_memory")]
    pub use super::database_local::*;
    pub use super::database_trait::*;
}
pub mod dynamodb_client {
    #[cfg(not(feature = "in_memory"))]
    pub use super::dynamodb_client_cloud::*;
    #[cfg(feature = "in_memory")]
    pub use super::dynamodb_client_local::*;
    pub use super::dynamodb_client_trait::*;
}
