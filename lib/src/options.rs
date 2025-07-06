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

    /// An optional user agent string to identify the sender.
    pub user_agent: Option<String>,
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

    /// Allows setting a custom user agent string
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

/// Options for receiving a secret.
#[derive(Default, Clone)]
pub struct SecretReceiveOptions {
    /// An optional observer to monitor the data transfer.
    pub observer: Option<Arc<dyn DataTransferObserver>>,

    /// An optional timeout for receiving the secret.
    pub timeout: Option<Duration>,

    /// An optional user agent string to identify the sender.
    pub user_agent: Option<String>,
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

    /// Allows setting a custom user agent string
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
    ///
    /// Allows setting an observer to monitor the data transfer.
    pub fn with_observer(mut self, observer: Arc<dyn DataTransferObserver>) -> Self {
        self.observer = Some(observer);
        self
    }
}
