use std::sync::Arc;
use std::time::Duration;

use crate::observer::DataTransferObserver;

#[derive(Default, Clone)]
pub struct SecretSendOptions {
    pub observer: Option<Arc<dyn DataTransferObserver>>,
    pub chunk_size: Option<usize>,
    pub timeout: Option<Duration>,
}

impl SecretSendOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_observer(mut self, observer: Arc<dyn DataTransferObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

#[derive(Default, Clone)]
pub struct SecretGetOptions {
    pub chunk_size: Option<usize>,
    pub timeout: Option<Duration>,
}

impl SecretGetOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
