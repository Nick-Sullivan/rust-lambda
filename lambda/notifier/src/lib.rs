pub mod dependency_injection;
mod notifier_cloud;
mod notifier_local;
mod notifier_trait;
pub mod notifier {
    #[cfg(not(feature = "in_memory"))]
    pub use super::notifier_cloud::*;
    #[cfg(feature = "in_memory")]
    pub use super::notifier_local::*;
    pub use super::notifier_trait::*;
}
