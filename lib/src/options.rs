use std::sync::Arc;
use std::time::Duration;

use crate::observer::DataTransferObserver;

/// Options for sending a secret.
#[derive(Default, Clone)]
pub struct SecretSendOptions {
    /// An optional observer to monitor the data transfer.
    pub observer: Option<Arc<dyn DataTransferObserver>>,

    /// An optional chunk size for sending the secret.
    pub chunk_size: Option<usize>,

    /// An optional timeout for sending the secret.
    pub timeout: Option<Duration>,
}

impl SecretSendOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows setting an observer to monitor the data transfer.
    pub fn with_observer(mut self, observer: Arc<dyn DataTransferObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Allows setting a custom chunk size for sending a secret.
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Allows setting a custom timeout for sending a secret.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Options for receiving a secret.
#[derive(Default, Clone)]
pub struct SecretReceiveOptions {
    pub chunk_size: Option<usize>,
    pub timeout: Option<Duration>,
}

impl SecretReceiveOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows setting a custom timeout for receiving a secret.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
