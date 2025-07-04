use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;

use crate::client::{Client, ClientError};
use crate::models::{PostSecretRequest, PostSecretResponse};

const SHORT_SECRET_PATH: &str = "s";
const API_SECRET_PATH: &str = "api/v1/secret";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = "hakanai-client";

#[derive(Debug)]
pub struct WebClient {
    web_client: reqwest::Client,
}

impl WebClient {
    /// Creates a new instance of `WebClient`.
    pub fn new() -> Self {
        WebClient {
            web_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Client<String> for WebClient {
    async fn send_secret(
        &self,
        base_url: Url,
        data: String,
        ttl: Duration,
        token: String,
    ) -> Result<Url, ClientError> {
        let url = base_url.join(API_SECRET_PATH)?;
        let req = PostSecretRequest::new(data, ttl);

        let resp = self
            .web_client
            .post(url.to_string())
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .json(&req)
            .send()
            .await?;
        if resp.status() != reqwest::StatusCode::OK {
            let mut err_msg = format!("HTTP error: {}", resp.status());

            if let Ok(body) = resp.text().await {
                err_msg += &format!("\n{body}");
            }

            return Err(ClientError::Http(err_msg));
        }

        let res = resp.json::<PostSecretResponse>().await?;

        let secret_url = base_url.join(&format!("{}/{}", SHORT_SECRET_PATH, res.id))?;
        Ok(secret_url)
    }

    async fn receive_secret(&self, url: Url) -> Result<String, ClientError> {
        if !url.path().starts_with(&format!("/{SHORT_SECRET_PATH}/"))
            && !url.path().starts_with(&format!("/{API_SECRET_PATH}/"))
        {
            return Err(ClientError::Custom("Invalid API path".to_string()));
        }

        let resp = self
            .web_client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await?;
        if resp.status() != reqwest::StatusCode::OK {
            let mut err_msg = format!("HTTP error: {}", resp.status());

            if let Ok(body) = resp.text().await {
                err_msg += &format!("\n{body}");
            }

            return Err(ClientError::Http(err_msg));
        }

        let secret = resp.text().await?;
        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;
    use url::Url;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_send_secret_success() {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let secret_id = Uuid::new_v4();
        let _m = server
            .mock("POST", "/api/v1/secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"id":"{secret_id}"}}"#))
            .create_async()
            .await;

        let base_url = Url::parse(&server.url()).unwrap();
        let result = client
            .send_secret(
                base_url.clone(),
                "test_secret".to_string(),
                Duration::from_secs(3600),
                "".to_string(),
            )
            .await;

        if let Err(e) = &result {
            eprintln!("Error in test_send_secret_success: {e:?}");
        }
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), format!("{base_url}s/{secret_id}"));
    }

    #[tokio::test]
    async fn test_send_secret_server_error() {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let _m = server
            .mock("POST", "/api/v1/secret")
            .with_status(500)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url()).unwrap();
        let result = client
            .send_secret(
                base_url,
                "test_secret".to_string(),
                Duration::from_secs(3600),
                "".to_string(),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_receive_secret_success() {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let secret_id = Uuid::new_v4();
        let secret_data = "my_secret_data";

        let _m = server
            .mock("GET", format!("/s/{secret_id}").as_str())
            .with_status(200)
            .with_body(secret_data)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url()).unwrap();
        let url = base_url.join(&format!("/s/{secret_id}")).unwrap();
        let result = client.receive_secret(url).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), secret_data);
    }

    #[tokio::test]
    async fn test_receive_secret_not_found() {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let secret_id = Uuid::new_v4();

        let _m = server
            .mock("GET", format!("/s/{secret_id}").as_str())
            .with_status(404)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url()).unwrap();
        let url = base_url.join(&format!("/s/{secret_id}")).unwrap();
        let result = client.receive_secret(url).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_secret_invalid_json_response() {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let _m = server
            .mock("POST", "/api/v1/secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": "json"}"#)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url()).unwrap();
        let result = client
            .send_secret(
                base_url,
                "test_secret".to_string(),
                Duration::from_secs(3600),
                "".to_string(),
            )
            .await;

        assert!(result.is_err());
    }
}
