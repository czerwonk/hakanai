use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;

use crate::client::{Client, ClientError};

/// A client that wraps another `Client` to provide cryptographic functionalities.
///
/// This struct is responsible for encrypting data before sending and decrypting
/// it upon reception, ensuring that secrets are transmitted securely.
pub struct CryptoClient {
    inner_client: Box<dyn Client>,
}

impl CryptoClient {
    /// Creates a new instance of `CryptoClient`.
    pub fn new(inner_client: Box<dyn Client>) -> Self {
        CryptoClient { inner_client }
    }
}

#[async_trait]
impl Client for CryptoClient {
    async fn send_secret(
        &self,
        base_url: Url,
        data: String,
        ttl: Duration,
    ) -> Result<Url, ClientError> {
        self.inner_client.send_secret(base_url, data, ttl).await
    }

    async fn receive_secret(&self, url: Url) -> Result<String, ClientError> {
        self.inner_client.receive_secret(url).await
    }
}
