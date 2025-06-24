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

    #[error("HTTP error: {0}")]
    Http(String),
}

#[derive(Debug)]
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
        let url = format!("{}api/secret", self.base_url);
        let req = PostSecretRequest::new(data, ttl);

        let resp = self.web_client.post(&url).json(&req).send().await?;
        if resp.status() != reqwest::StatusCode::OK {
            let mut err_msg = format!("HTTP error: {}", resp.status());

            if let Ok(body) = resp.text().await {
                err_msg += &format!("\n{}", body);
            }

            return Err(ClientError::Http(err_msg));
        }

        let res = resp.json::<PostSecretResponse>().await?;

        let secret_url = Url::parse(&format!("{}/secret/{}", self.base_url, res.id))?;
        Ok(secret_url)
    }

    async fn receive_secret(&self, id: Uuid) -> Result<String, ClientError> {
        let url = format!("{}api/secret/{}", self.base_url, id);

        let resp = self.web_client.get(url).send().await?;
        if resp.status() != reqwest::StatusCode::OK {
            let mut err_msg = format!("HTTP error: {}", resp.status());

            if let Ok(body) = resp.text().await {
                err_msg += &format!("\n{}", body);
            }

            return Err(ClientError::Http(err_msg));
        }

        let secret = resp.text().await?;
        Ok(secret)
    }
}

/// Creates a new web client.
///
/// This function returns a new instance of `WebClient` that implements the `Client` trait.
pub fn new(base_url: Url) -> impl Client {
    WebClient::new(base_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use std::time::Duration;

    #[test]
    fn test_web_client_new() {
        let base_url = Url::parse("https://example.com").unwrap();
        let client = WebClient::new(base_url.clone());

        assert_eq!(client.base_url.as_str(), "https://example.com/");
    }

    #[tokio::test]
    async fn test_send_secret_success() {
        let mut server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url()).unwrap();
        let client = WebClient::new(base_url.clone());

        let secret_id = Uuid::new_v4();
        let _m = server
            .mock("POST", "/api/secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"id":"{}"}}"#, secret_id))
            .create_async()
            .await;

        let result = client
            .send_secret("test_secret".to_string(), Duration::from_secs(3600))
            .await;

        if let Err(e) = &result {
            eprintln!("Error in test_send_secret_success: {:?}", e);
        }
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), format!("{}/secret/{}", base_url, secret_id));
    }

    #[tokio::test]
    async fn test_send_secret_server_error() {
        let mut server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url()).unwrap();
        let client = WebClient::new(base_url);

        let _m = server
            .mock("POST", "/api/secret")
            .with_status(500)
            .create_async()
            .await;

        let result = client
            .send_secret("test_secret".to_string(), Duration::from_secs(3600))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_receive_secret_success() {
        let mut server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url()).unwrap();
        let client = WebClient::new(base_url);

        let secret_id = Uuid::new_v4();
        let secret_data = "my_secret_data";

        let _m = server
            .mock("GET", format!("/api/secret/{}", secret_id).as_str())
            .with_status(200)
            .with_body(secret_data)
            .create_async()
            .await;

        let result = client.receive_secret(secret_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), secret_data);
    }

    #[tokio::test]
    async fn test_receive_secret_not_found() {
        let mut server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url()).unwrap();
        let client = WebClient::new(base_url);

        let secret_id = Uuid::new_v4();

        dbg!(&client);
        let _m = server
            .mock("GET", format!("/api/secret/{}", secret_id).as_str())
            .with_status(404)
            .create_async()
            .await;

        let result = client.receive_secret(secret_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_secret_invalid_json_response() {
        let mut server = mockito::Server::new_async().await;
        let base_url = Url::parse(&server.url()).unwrap();
        let client = WebClient::new(base_url);

        let _m = server
            .mock("POST", "/api/secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": "json"}"#)
            .create_async()
            .await;

        let result = client
            .send_secret("test_secret".to_string(), Duration::from_secs(3600))
            .await;

        assert!(result.is_err());
    }
}
