use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::models::{PostSecretRequest, PostSecretResponse};

/// Defines the asynchronous interface for a client that can send and receive secrets.
#[async_trait]
pub trait Client: Send + Sync {
    /// Sends a secret to be stored.
    ///
    /// # Arguments
    ///
    /// * `data` - The secret data to be sent.
    /// * `ttl` - The time-to-live for the secret.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Url)` containing the URL of the stored secret.
    /// - `Err(ClientError)` with an error message if the operation fails.
    async fn send_secret(&self, data: String, ttl: Duration) -> Result<Url, ClientError>;

    /// Retrieves a secret from the store using its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the secret to be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)` containing the secret data.
    /// - `Err(ClientError)` with an error message if the secret is not found or another error occurs.
    async fn receive_secret(&self, id: Uuid) -> Result<String, ClientError>;
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("web request failed")]
    Web(#[from] reqwest::Error),

    #[error("parsing JSON failed")]
    Json(#[from] serde_json::Error),

    #[error("invalid URL")]
    Url(#[from] url::ParseError),
}

pub struct WebClient {
    web_client: reqwest::Client,
    base_url: Url,
}

impl WebClient {
    /// Creates a new instance of `WebClient`.
    pub fn new(base_url: Url) -> Self {
        WebClient {
            web_client: reqwest::Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl Client for WebClient {
    async fn send_secret(&self, data: String, ttl: Duration) -> Result<Url, ClientError> {
        let url = format!("{}/api/secret", self.base_url);
        let req = PostSecretRequest::new(data, ttl);

        let resp = self
            .web_client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<PostSecretResponse>()
            .await?;

        let secret_url = Url::parse(&format!("{}/secret/{}", self.base_url, resp.id))?;
        Ok(secret_url)
    }

    async fn receive_secret(&self, id: Uuid) -> Result<String, ClientError> {
        let url = format!("{}/api/secret/{}", self.base_url, id);
        let secret = self.web_client.get(url).send().await?.text().await?;

        Ok(secret)
    }
}

/// Creates a new web client.
///
/// This function returns a new instance of `WebClient` that implements the `Client` trait.
pub fn new(base_url: Url) -> impl Client {
    WebClient::new(base_url)
}
