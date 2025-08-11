use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Body, Url};
use uuid::Uuid;

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
impl Client<Vec<u8>> for WebClient {
    async fn send_secret(
        &self,
        base_url: Url,
        data: Vec<u8>,
        ttl: Duration,
        token: String,
        opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        let url = base_url.join(API_SECRET_PATH)?;

        let secret = String::from_utf8(data)?;
        let req = PostSecretRequest::new(secret, ttl);

        let opt = opts.unwrap_or_default();

        let (body, content_length) = self.post_secret_body_from_req(req, &opt)?;

        let timeout = opt.timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
        let user_agent = opt.user_agent.unwrap_or(DEFAULT_USER_AGENT.to_string());
        let request_id = Uuid::new_v4().to_string();

        let mut req = self
            .web_client
            .post(url.to_string())
            .header("User-Agent", user_agent)
            .header("Content-Type", "application/json")
            .header("Content-Length", content_length.to_string())
            .header("X-Request-Id", request_id)
            .body(body)
            .timeout(timeout);

        if !token.is_empty() {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await?;

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
    ) -> Result<Vec<u8>, ClientError> {
        if !url.path().starts_with(&format!("/{SHORT_SECRET_PATH}/"))
            && !url.path().starts_with(&format!("/{API_SECRET_PATH}/"))
        {
            return Err(ClientError::Custom("Invalid API path".to_string()));
        }

        let opt = opts.unwrap_or_default();
        let user_agent = opt.user_agent.unwrap_or(DEFAULT_USER_AGENT.to_string());
        let timeout = opt.timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
        let request_id = Uuid::new_v4().to_string();

        let mut resp = self
            .web_client
            .get(url)
            .header("User-Agent", user_agent)
            .header("X-Request-Id", request_id)
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

        let observer = opt.observer.clone();
        let secret = self.read_body_in_chunks(&mut resp, observer).await?;

        Ok(secret)
    }
}

impl WebClient {
    async fn read_body_in_chunks(
        &self,
        resp: &mut reqwest::Response,
        observer: Option<Arc<dyn DataTransferObserver>>,
    ) -> Result<Vec<u8>, ClientError> {
        let total_size = resp.content_length().unwrap_or(0);
        if total_size == 0 {
            return Err(ClientError::Custom(
                "Response body is empty or content length is not set".to_string(),
            ));
        }

        let mut result = Vec::with_capacity(total_size as usize);
        let mut bytes_read = 0u64;

        while let Some(chunk) = resp.chunk().await? {
            result.extend_from_slice(&chunk);
            bytes_read += chunk.len() as u64;

            if let Some(ref obs) = observer {
                obs.on_progress(bytes_read, total_size).await;
            }
        }

        Ok(result)
    }

    fn post_secret_body_from_req(
        &self,
        req: PostSecretRequest,
        opts: &SecretSendOptions,
    ) -> Result<(Body, usize), ClientError> {
        let json_bytes = serde_json::to_vec(&req)?;
        let json_len = json_bytes.len();

        let chunk_size = opts.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        if chunk_size == 0 {
            return Err(ClientError::Custom(
                "Chunk size must be greater than 0".to_string(),
            ));
        }

        let mut bytes_uploaded = 0u64;
        let upload_observer = self.upload_observer.clone();
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

    use std::error::Error;
    use std::time::Duration;
    use url::Url;
    use uuid::Uuid;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[tokio::test]
    async fn test_send_secret_success() -> Result<()> {
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

        let base_url = Url::parse(&server.url())?;
        let result = client
            .send_secret(
                base_url.clone(),
                b"test_secret".to_vec(),
                Duration::from_secs(3600),
                "".to_string(),
                None,
            )
            .await;

        let url = result?;
        assert_eq!(url.as_str(), format!("{base_url}s/{secret_id}"));
        Ok(())
    }

    #[tokio::test]
    async fn test_send_secret_server_error() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let _m = server
            .mock("POST", "/api/v1/secret")
            .with_status(500)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url())?;
        let result = client
            .send_secret(
                base_url,
                b"test_secret".to_vec(),
                Duration::from_secs(3600),
                "".to_string(),
                None,
            )
            .await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_success() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let secret_id = Uuid::new_v4();
        let secret_data = b"my_secret_data";

        let _m = server
            .mock("GET", format!("/s/{secret_id}").as_str())
            .with_status(200)
            .with_body(secret_data)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url())?;
        let url = base_url.join(&format!("/s/{secret_id}"))?;
        let result = client.receive_secret(url, None).await;

        let data = result?;
        assert_eq!(data, secret_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_not_found() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let secret_id = Uuid::new_v4();

        let _m = server
            .mock("GET", format!("/s/{secret_id}").as_str())
            .with_status(404)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url())?;
        let url = base_url.join(&format!("/s/{secret_id}"))?;
        let result = client.receive_secret(url, None).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_send_secret_invalid_json_response() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        let client = WebClient::new();

        let _m = server
            .mock("POST", "/api/v1/secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": "json"}"#)
            .create_async()
            .await;

        let base_url = Url::parse(&server.url())?;
        let result = client
            .send_secret(
                base_url,
                b"test_secret".to_vec(),
                Duration::from_secs(3600),
                "".to_string(),
                None,
            )
            .await;

        assert!(result.is_err());
        Ok(())
    }
}
