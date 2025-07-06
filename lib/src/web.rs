use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Body, Url};

use crate::client::{Client, ClientError};
use crate::models::{PostSecretRequest, PostSecretResponse};
use crate::observer::DataTransferObserver;
use crate::options::{SecretReceiveOptions, SecretSendOptions};

const SHORT_SECRET_PATH: &str = "s";
const API_SECRET_PATH: &str = "api/v1/secret";
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_USER_AGENT: &str = "hakanai-client";
const DEFAULT_CHUNK_SIZE: usize = 8192; // 8 KB

pub struct WebClient {
    web_client: reqwest::Client,
    upload_observer: Option<Arc<dyn DataTransferObserver>>,
}

impl WebClient {
    /// Creates a new instance of `WebClient`.
    pub fn new() -> Self {
        WebClient {
            web_client: reqwest::Client::new(),
            upload_observer: None,
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
        opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        let url = base_url.join(API_SECRET_PATH)?;
        let req = PostSecretRequest::new(data, ttl);

        let opt = opts.unwrap_or_default();

        let (body, content_length) = self.post_secret_body_from_req(req, &opt)?;

        let timeout = opt.timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
        let user_agent = opt.user_agent.unwrap_or(DEFAULT_USER_AGENT.to_string());

        let resp = self
            .web_client
            .post(url.to_string())
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", user_agent)
            .header("Content-Type", "application/json")
            .header("Content-Length", content_length.to_string())
            .timeout(timeout)
            .body(body)
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

    async fn receive_secret(
        &self,
        url: Url,
        opts: Option<SecretReceiveOptions>,
    ) -> Result<String, ClientError> {
        if !url.path().starts_with(&format!("/{SHORT_SECRET_PATH}/"))
            && !url.path().starts_with(&format!("/{API_SECRET_PATH}/"))
        {
            return Err(ClientError::Custom("Invalid API path".to_string()));
        }

        let opt = opts.unwrap_or_default();
        let timeout = opt.timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);

        let resp = self
            .web_client
            .get(url)
            .header("User-Agent", DEFAULT_USER_AGENT)
            .timeout(timeout)
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

impl WebClient {
    fn post_secret_body_from_req(
        &self,
        req: PostSecretRequest,
        opts: &SecretSendOptions,
    ) -> Result<(Body, usize), ClientError> {
        let json_bytes = serde_json::to_vec(&req)?;
        let json_len = json_bytes.len();
        let mut bytes_uploaded = 0u64;

        let upload_observer = self.upload_observer.clone();

        let chunk_size = opts.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        let stream = async_stream::stream! {
            let mut offset = 0;

            while offset < json_len {
                let end = std::cmp::min(offset + chunk_size, json_bytes.len());
                let chunk = Bytes::copy_from_slice(&json_bytes[offset..end]);
                bytes_uploaded += chunk.len() as u64;

                if let Some(ref observer) = upload_observer {
                    observer.on_progress(bytes_uploaded, json_len as u64).await;
                }

                yield Ok::<_, std::io::Error>(chunk);
                offset = end;
            }
        };

        Ok((Body::wrap_stream(stream), json_len))
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
                None,
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
                None,
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
        let result = client.receive_secret(url, None).await;

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
        let result = client.receive_secret(url, None).await;
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
                None,
            )
            .await;

        assert!(result.is_err());
    }
}
