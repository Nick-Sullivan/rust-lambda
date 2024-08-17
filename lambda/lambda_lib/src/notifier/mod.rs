mod notifier_cloud;
mod notifier_local;
mod notifier_trait;
pub mod notifier {
    #[cfg(not(test))]
    pub use super::notifier_cloud::*;
    #[cfg(test)]
    pub use super::notifier_local::*;
    pub use super::notifier_trait::*;
}
