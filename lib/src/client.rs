use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::crypto::CryptoClient;
use crate::models::Payload;
use crate::web::WebClient;

/// Defines the asynchronous interface for a client that can send and receive secrets.
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
    async fn receive_secret(&self, url: Url) -> Result<T, ClientError>;
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

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),
}

/// A client for sending and receiving `Payload` objects.
///
/// This client acts as a layer over a `Client<String>`, handling the serialization
/// and deserialization of `Payload` objects to and from JSON strings.
pub struct SecretClient {
    client: Box<dyn Client<String>>,
}

#[async_trait]
impl Client<Payload> for SecretClient {
    async fn send_secret(
        &self,
        base_url: Url,
        payload: Payload,
        ttl: Duration,
        token: String,
    ) -> Result<Url, ClientError> {
        let data = serde_json::to_string(&payload)?;
        let url = self.client.send_secret(base_url, data, ttl, token).await?;
        Ok(url)
    }

    async fn receive_secret(&self, url: Url) -> Result<Payload, ClientError> {
        let data = self.client.receive_secret(url).await?;
        let payload: Payload = serde_json::from_str(&data)?;
        Ok(payload)
    }
}

/// Creates a new client instance.
///
/// This function constructs a default client implementation, which is a `CryptoClient`
/// wrapping a `WebClient`. This setup provides end-to-end encryption for secrets.
pub fn new() -> impl Client<Payload> {
    SecretClient {
        client: Box::new(CryptoClient::new(Box::new(WebClient::new()))),
    }
}
