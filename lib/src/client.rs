use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::web::WebClient;

/// Defines the asynchronous interface for a client that can send and receive secrets.
#[async_trait]
pub trait Client: Send + Sync {
    /// Sends a secret to be stored.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the service.
    /// * `data` - The secret data to be sent.
    /// * `ttl` - The time-to-live for the secret.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Url)` containing the URL of the stored secret.
    /// - `Err(ClientError)` with an error message if the operation fails.
    async fn send_secret(
        &self,
        base_url: Url,
        data: String,
        ttl: Duration,
    ) -> Result<Url, ClientError>;

    /// Retrieves a secret from the store using its URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the secret to be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)` containing the secret data.
    /// - `Err(ClientError)` with an error message if the secret is not found or another error occurs.
    async fn receive_secret(&self, url: Url) -> Result<String, ClientError>;
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("web request failed")]
    Web(#[from] reqwest::Error),

    #[error("parsing JSON failed")]
    Json(#[from] serde_json::Error),

    #[error("invalid URL")]
    Url(#[from] url::ParseError),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Client error: {0}")]
    Custom(String),
}

/// Creates a new web client.
///
/// This function returns a new instance of `WebClient` that implements the `Client` trait.
pub fn new() -> impl Client {
    WebClient::new()
}
