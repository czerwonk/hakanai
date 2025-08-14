// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::crypto::CryptoClient;
use crate::models::Payload;
use crate::options::{SecretReceiveOptions, SecretSendOptions};
use crate::web::WebClient;

/// Defines the asynchronous interface for a client that can send and receive secrets.
///
/// This trait represents the core API for secret operations. The library provides
/// a default implementation via `client::new()`, but users can create wrapper
/// clients that add additional functionality around the core client.
///
/// # Examples
///
/// ## Adding Validation to Client Operations
///
/// ```
/// use hakanai_lib::{client, client::{Client, ClientError}, models::Payload};
/// use hakanai_lib::options::{SecretSendOptions, SecretReceiveOptions};
/// use async_trait::async_trait;
/// use url::Url;
/// use std::time::Duration;
///
/// // Wrapper that adds input validation to client operations
/// struct ValidatingClient {
///     inner: Box<dyn Client<Payload>>,
///     max_size: usize,
/// }
///
/// impl ValidatingClient {
///     fn new(inner: Box<dyn Client<Payload>>, max_size: usize) -> Self {
///         Self { inner, max_size }
///     }
/// }
///
/// #[async_trait]
/// impl Client<Payload> for ValidatingClient {
///     async fn send_secret(
///         &self,
///         base_url: Url,
///         payload: Payload,
///         ttl: Duration,
///         token: String,
///         opts: Option<SecretSendOptions>,
///     ) -> Result<Url, ClientError> {
///         // Validate payload size before sending
///         if payload.data.len() > self.max_size {
///             return Err(ClientError::Custom(format!(
///                 "Payload size {} exceeds maximum {}",
///                 payload.data.len(),
///                 self.max_size
///             )));
///         }
///
///         // Validate filename for security
///         if let Some(ref filename) = payload.filename {
///             if filename.contains("..") || filename.starts_with('/') {
///                 return Err(ClientError::Custom(
///                     "Invalid filename: path traversal detected".to_string()
///                 ));
///             }
///         }
///
///         // Validation passed, proceed with sending
///         self.inner.send_secret(base_url, payload, ttl, token, opts).await
///     }
///
///     async fn receive_secret(
///         &self,
///         url: Url,
///         opts: Option<SecretReceiveOptions>,
///     ) -> Result<Payload, ClientError> {
///         // Pass through to inner client
///         self.inner.receive_secret(url, opts).await
///     }
/// }
///
/// // Usage: wrap the default client with validation
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let base_client = client::new();
/// let validating_client = ValidatingClient::new(Box::new(base_client), 1024 * 1024); // 1MB limit
///
/// let payload = Payload {
///     data: "test secret".to_string(),
///     filename: None,
/// };
///
/// // This will validate before sending
/// let url = validating_client.send_secret(
///     Url::parse("https://api.example.com")?,
///     payload,
///     Duration::from_secs(3600),
///     "token".to_string(),
///     None,
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Client<T>: Send + Sync {
    /// Sends a secret to be stored.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the service.
    /// * `data` - The secret data to be sent.
    /// * `ttl` - The time-to-live for the secret.
    /// * `token` - The authentication token.
    /// * `opts` - Optional options for sending the secret
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Url)` containing the URL of the stored secret.
    /// - `Err(ClientError)` with an error message if the operation fails.
    async fn send_secret(
        &self,
        base_url: Url,
        payload: T,
        ttl: Duration,
        token: String,
        opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError>;

    /// Retrieves a secret from the store using its URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the secret to be retrieved.
    /// * `opts` - Optional options for retrieving the secret
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)` containing the secret data.
    /// - `Err(ClientError)` with an error message if the secret is not found or another error occurs.
    async fn receive_secret(
        &self,
        url: Url,
        opts: Option<SecretReceiveOptions>,
    ) -> Result<T, ClientError>;
}

/// Represents errors that can occur during client operations.
///
/// This enum covers all possible error cases when sending or receiving secrets,
/// including network errors, parsing errors, and cryptographic failures.
///
/// # Examples
///
/// ```
/// use hakanai_lib::{client, client::{Client, ClientError}, models::Payload};
/// use std::time::Duration;
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = client::new();
/// let payload = Payload::from_bytes(b"Test secret", None);
///
/// match client.send_secret(
///     Url::parse("https://api.example.com")?,
///     payload,
///     Duration::from_secs(3600),
///     "auth-token".to_string(),
///     None,
/// ).await {
///     Ok(url) => println!("Secret stored at: {}", url),
///     Err(ClientError::Web(e)) => eprintln!("Network error: {}", e),
///     Err(ClientError::Http(msg)) => eprintln!("Server error: {}", msg),
///     Err(ClientError::CryptoError(msg)) => eprintln!("Decryption failed: {}", msg),
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Error)]
pub enum ClientError {
    /// Network request failed.
    ///
    /// This error occurs when the underlying HTTP client (reqwest) encounters
    /// a network-level error such as connection timeout, DNS resolution failure,
    /// or inability to establish a connection.
    #[error("web request failed")]
    Web(#[from] reqwest::Error),

    /// JSON parsing or serialization failed.
    ///
    /// This error occurs when the response from the server cannot be parsed
    /// as valid JSON, or when serializing the payload to JSON fails.
    #[error("parsing JSON failed")]
    Json(#[from] serde_json::Error),

    /// URL parsing failed.
    ///
    /// This error occurs when constructing or parsing URLs, typically when
    /// the base URL or secret URL is malformed.
    #[error("invalid URL")]
    Url(#[from] url::ParseError),

    /// HTTP-level error from the server.
    ///
    /// This error represents HTTP status code errors (4xx, 5xx) returned by
    /// the server, with the error message containing details about the failure.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Custom client error.
    ///
    /// This is a catch-all error for client-specific failures that don't
    /// fit into other categories.
    #[error("Client error: {0}")]
    Custom(String),

    /// Cryptographic error.
    #[error("crypto error")]
    CryptoError(String),

    /// Error converting bytes to UTF-8.
    #[error("UTF-8 decoding error")]
    Utf8DecodeError(#[from] std::string::FromUtf8Error),

    /// Base64 decoding error.
    #[error("base64 decoding error")]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("decrypted data does not match expected hash")]
    HashValidationError(),
}

impl From<aes_gcm::Error> for ClientError {
    fn from(err: aes_gcm::Error) -> Self {
        ClientError::CryptoError(format!("AES-GCM error: {err:?}"))
    }
}

/// Creates a new client instance with the default configuration.
///
/// This function constructs a layered client stack that provides:
/// - HTTP communication via `WebClient`
/// - AES-256-GCM encryption via `CryptoClient`
/// - Automatic `Payload` serialization via `SecretClient`
///
/// # Examples
///
/// ```no_run
/// use hakanai_lib::{client, client::Client, models::Payload};
/// use std::time::Duration;
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create the default client
/// let client = client::new();
///
/// // Send a secret
/// let url = client.send_secret(
///     Url::parse("https://api.example.com")?,
///     Payload {
///         data: "my secret data".to_string(),
///         filename: None,
///     },
///     Duration::from_secs(3600),
///     "auth-token".to_string(),
///     None,
/// ).await?;
///
/// // The URL contains the encryption key and hash in the fragment (#key:hash)
/// println!("Share this URL: {}", url);
/// # Ok(())
/// # }
/// ```
pub fn new() -> impl Client<Payload> {
    CryptoClient::new(Box::new(WebClient::new()))
}
