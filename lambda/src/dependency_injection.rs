use crate::storage;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[cfg(not(feature = "is_local"))]
use storage::database_cloud::NameCounter;
#[cfg(feature = "is_local")]
use storage::database_local::NameCounter;

pub static DATABASE: Lazy<Mutex<NameCounter>> = Lazy::new(|| {
    let db = NameCounter::new();
    Mutex::new(db)
});
