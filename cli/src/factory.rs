use std::sync::Arc;

use anyhow::Result;

use hakanai_lib::observer::DataTransferObserver;
use hakanai_lib::{client::Client, models::Payload};

use crate::observer::ProgressObserver;

// Factory for dependency injection
pub trait Factory: Send + Sync {
    // Creates a new client instance.
    fn new_client(&self) -> impl Client<Payload>;

    /// Creates a new observer instance with the given label.
    fn new_observer(&self, label: &str) -> Result<Arc<dyn DataTransferObserver>>;
}

/// Application factory that implements the Factory trait for the CLI application.
pub struct AppFactory {}

impl Factory for AppFactory {
    fn new_client(&self) -> impl Client<Payload> {
        hakanai_lib::client::new()
    }

    fn new_observer(&self, label: &str) -> Result<Arc<dyn DataTransferObserver>> {
        Ok(Arc::new(ProgressObserver::new(label)?))
    }
}
