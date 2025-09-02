// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use crate::models::SecretRestrictions;
use crate::observer::DataTransferObserver;
use crate::utils::hashing;

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

    /// Optional access restrictions for the secret.
    pub restrictions: Option<SecretRestrictions>,
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

    /// Sets the access restrictions for the secret.
    pub fn with_restrictions(mut self, restrictions: SecretRestrictions) -> Self {
        self.restrictions = Some(restrictions);
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

    /// An optional passphrase hash required to access the secret.
    pub passphrase_hash: Option<String>,
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

    /// Sets a passphrase for accessing the secret
    pub fn with_passphrase(mut self, passphrase: &[u8]) -> Self {
        if passphrase.is_empty() {
            return self;
        }

        let hash = hashing::sha256_hex_from_bytes(passphrase);
        self.passphrase_hash = Some(hash);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Tests for SecretReceiveOptions passphrase functionality
    #[test]
    fn test_secret_receive_options_with_passphrase() {
        let opts = SecretReceiveOptions::new().with_passphrase(b"mypassword");

        assert!(
            opts.passphrase_hash.is_some(),
            "Passphrase hash should be set"
        );
        let hash = opts.passphrase_hash.unwrap();
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 characters long");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex digits"
        );
    }

    #[test]
    fn test_secret_receive_options_with_passphrase_empty() {
        let opts = SecretReceiveOptions::new().with_passphrase(b"");

        assert!(
            opts.passphrase_hash.is_none(),
            "Empty passphrase should be ignored"
        );
    }

    #[test]
    fn test_secret_receive_options_default_values() {
        let opts = SecretReceiveOptions::new();

        assert!(opts.observer.is_none(), "Default observer should be None");
        assert!(opts.timeout.is_none(), "Default timeout should be None");
        assert!(
            opts.user_agent.is_none(),
            "Default user agent should be None"
        );
        assert!(
            opts.passphrase_hash.is_none(),
            "Default passphrase hash should be None"
        );
    }

    #[test]
    fn test_secret_receive_options_builder_pattern() {
        let opts = SecretReceiveOptions::default()
            .with_timeout(Duration::from_secs(120))
            .with_user_agent("BuilderTest/1.0".to_string())
            .with_passphrase(b"builder_test");

        assert_eq!(
            opts.timeout,
            Some(Duration::from_secs(120)),
            "Builder pattern should set timeout"
        );
        assert_eq!(
            opts.user_agent,
            Some("BuilderTest/1.0".to_string()),
            "Builder pattern should set user agent"
        );
        assert!(
            opts.passphrase_hash.is_some(),
            "Builder pattern should set passphrase hash"
        );
    }
}
