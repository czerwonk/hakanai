// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use crate::observer::DataTransferObserver;

/// Options for sending a secret.
///
/// This struct provides a builder pattern for configuring how secrets are sent,
/// including progress monitoring, chunk sizes, timeouts, and user agent identification.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use std::time::Duration;
/// use hakanai_lib::options::SecretSendOptions;
/// use hakanai_lib::observer::DataTransferObserver;
///
/// # struct MyObserver;
/// # #[async_trait::async_trait]
/// # impl DataTransferObserver for MyObserver {
/// #     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {}
/// # }
///
/// // Create options with default settings
/// let default_opts = SecretSendOptions::new();
///
/// // Configure with custom settings using the builder pattern
/// let custom_opts = SecretSendOptions::new()
///     .with_observer(Arc::new(MyObserver))
///     .with_chunk_size(8192)
///     .with_timeout(Duration::from_secs(30))
///     .with_user_agent("MyApp/1.0".to_string());
/// ```
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

    /// An optional list of IP networks allowed to access the secret.
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,
}

impl SecretSendOptions {
    /// Creates a new `SecretSendOptions` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an observer to monitor the data transfer progress.
    pub fn with_observer(mut self, observer: Arc<dyn DataTransferObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Sets a custom chunk size for sending data.
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Sets a custom timeout for the send operation.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets a custom user agent string.
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Sets the allowed IPs for this secret.
    pub fn with_allowed_ips(mut self, allowed_ips: Vec<ipnet::IpNet>) -> Self {
        self.allowed_ips = Some(allowed_ips);
        self
    }
}

/// Options for receiving a secret.
///
/// This struct provides a builder pattern for configuring how secrets are received,
/// including progress monitoring, timeouts, and user agent identification.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use std::time::Duration;
/// use hakanai_lib::options::SecretReceiveOptions;
/// use hakanai_lib::observer::DataTransferObserver;
///
/// # struct DownloadMonitor;
/// # #[async_trait::async_trait]
/// # impl DataTransferObserver for DownloadMonitor {
/// #     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {}
/// # }
///
/// // Create options with default settings
/// let default_opts = SecretReceiveOptions::new();
///
/// // Configure with custom settings using the builder pattern
/// let custom_opts = SecretReceiveOptions::new()
///     .with_timeout(Duration::from_secs(120))
///     .with_user_agent("MyApp/1.0".to_string())
///     .with_observer(Arc::new(DownloadMonitor));
/// ```
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
    /// Creates a new `SecretReceiveOptions` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom timeout for the receive operation.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets a custom user agent string.
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Sets an observer to monitor the data transfer progress.
    pub fn with_observer(mut self, observer: Arc<dyn DataTransferObserver>) -> Self {
        self.observer = Some(observer);
        self
    }
}
