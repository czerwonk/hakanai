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

/// A trait for observing the progress of upload operations.
///
/// Implementors of this trait can receive real-time notifications about upload progress,
/// allowing for features like progress bars, bandwidth monitoring, or logging.
///
/// # Thread Safety
///
/// This trait requires `Send + Sync` to ensure it can be safely used across async tasks
/// and shared between threads.
#[async_trait::async_trait]
pub trait DataTransferObserver: Send + Sync {
    /// Called when data transfer progress is made.
    ///
    /// This method is invoked periodically during the data transfer process.
    ///
    /// # Arguments
    ///
    /// * `bytes_uploaded` - The total number of bytes transferred so far
    /// * `total_bytes` - The total size of the transfer in bytes
    ///
    /// # Notes
    ///
    /// - This method is called asynchronously and should not block for extended periods
    /// - The frequency of calls depends on the chunk size used
    /// - `bytes_uploaded` will always be ≤ `total_bytes`
    /// - The final call will have `bytes_uploaded == total_bytes`
    async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64);
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockClient {
        sent_data: Arc<Mutex<Option<String>>>,
        response_url: Option<Url>,
        response_data: Option<String>,
        fail_send: bool,
        fail_receive: bool,
    }

    impl MockClient {
        fn new() -> Self {
            Self {
                sent_data: Arc::new(Mutex::new(None)),
                response_url: Some(Url::parse("https://example.com/secret/123").unwrap()),
                response_data: Some(r#"{"data":"test data","filename":null}"#.to_string()),
                fail_send: false,
                fail_receive: false,
            }
        }

        fn with_response_data(mut self, data: &str) -> Self {
            self.response_data = Some(data.to_string());
            self
        }

        fn with_send_failure(mut self) -> Self {
            self.fail_send = true;
            self
        }

        fn with_receive_failure(mut self) -> Self {
            self.fail_receive = true;
            self
        }

        fn get_sent_data(&self) -> Option<String> {
            self.sent_data.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Client<String> for MockClient {
        async fn send_secret(
            &self,
            _base_url: Url,
            payload: String,
            _ttl: Duration,
            _token: String,
        ) -> Result<Url, ClientError> {
            if self.fail_send {
                return Err(ClientError::Custom("Send failed".to_string()));
            }
            *self.sent_data.lock().unwrap() = Some(payload);
            self.response_url
                .clone()
                .ok_or(ClientError::Custom("No response URL".to_string()))
        }

        async fn receive_secret(&self, _url: Url) -> Result<String, ClientError> {
            if self.fail_receive {
                return Err(ClientError::Custom("Receive failed".to_string()));
            }
            self.response_data
                .clone()
                .ok_or(ClientError::Custom("No response data".to_string()))
        }
    }

    #[tokio::test]
    async fn test_secret_client_send_text_payload() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let payload = Payload {
            data: "Hello, World!".to_string(),
            filename: None,
        };

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "https://example.com/secret/123");

        // Verify the payload was serialized correctly
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();
        assert_eq!(sent_payload.data, "Hello, World!");
        assert_eq!(sent_payload.filename, None);
    }

    #[tokio::test]
    async fn test_secret_client_send_file_payload() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let payload = Payload {
            data: base64::prelude::BASE64_STANDARD.encode(b"Binary file content"),
            filename: Some("document.pdf".to_string()),
        };

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());

        // Verify the payload was serialized correctly
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();
        assert_eq!(
            sent_payload.data,
            base64::prelude::BASE64_STANDARD.encode(b"Binary file content")
        );
        assert_eq!(sent_payload.filename, Some("document.pdf".to_string()));
    }

    #[tokio::test]
    async fn test_secret_client_receive_text_payload() {
        let response_json = r#"{"data":"Hello from server","filename":null}"#;
        let mock_client = MockClient::new().with_response_data(response_json);
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url).await;

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.data, "Hello from server");
        assert_eq!(payload.filename, None);
    }

    #[tokio::test]
    async fn test_secret_client_receive_file_payload() {
        let response_json = r#"{"data":"U29tZSBiaW5hcnkgZGF0YQ==","filename":"test.bin"}"#;
        let mock_client = MockClient::new().with_response_data(response_json);
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url).await;

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.data, "U29tZSBiaW5hcnkgZGF0YQ==");
        assert_eq!(payload.filename, Some("test.bin".to_string()));

        // Verify the data can be decoded
        let decoded = payload.decode_bytes().unwrap();
        assert_eq!(decoded, b"Some binary data");
    }

    #[tokio::test]
    async fn test_secret_client_send_error_propagation() {
        let mock_client = MockClient::new().with_send_failure();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let payload = Payload {
            data: "test".to_string(),
            filename: None,
        };

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Send failed"),
            _ => panic!("Expected Custom error"),
        }
    }

    #[tokio::test]
    async fn test_secret_client_receive_error_propagation() {
        let mock_client = MockClient::new().with_receive_failure();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Receive failed"),
            _ => panic!("Expected Custom error"),
        }
    }

    #[tokio::test]
    async fn test_secret_client_invalid_json_response() {
        let invalid_json = r#"{"data": "test", invalid json"#;
        let mock_client = MockClient::new().with_response_data(invalid_json);
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Json(_) => (),
            _ => panic!("Expected Json error"),
        }
    }

    #[tokio::test]
    async fn test_secret_client_payload_roundtrip() {
        // Test that a payload can be serialized and deserialized correctly
        let original_payload = Payload {
            data: base64::prelude::BASE64_STANDARD.encode(b"Test binary data"),
            filename: Some("test.dat".to_string()),
        };

        let json = serde_json::to_string(&original_payload).unwrap();
        let mock_client = MockClient::new().with_response_data(&json);
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url).await;

        assert!(result.is_ok());
        let received_payload = result.unwrap();
        assert_eq!(received_payload.data, original_payload.data);
        assert_eq!(received_payload.filename, original_payload.filename);

        // Verify the binary data is preserved
        let original_bytes = original_payload.decode_bytes().unwrap();
        let received_bytes = received_payload.decode_bytes().unwrap();
        assert_eq!(original_bytes, received_bytes);
    }

    #[tokio::test]
    async fn test_new_creates_correct_client_stack() {
        // This test verifies that the new() function creates the expected client stack
        let client = new();

        // We can test that it implements the Client<Payload> trait
        let payload = Payload {
            data: "test".to_string(),
            filename: None,
        };

        // This will fail since we don't have a real server, but it proves the client is constructed correctly
        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = client.send_secret(base_url, payload, ttl, token).await;

        // We expect this to fail with a network error since there's no real server
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_client_send_from_bytes() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Create payload using from_bytes
        let binary_data = b"This is raw binary data \x00\x01\x02\xFF";
        let payload = Payload::from_bytes(binary_data, Some("binary.dat".to_string()));

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());

        // Verify the payload was serialized correctly
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();

        // Verify that the data is base64 encoded
        assert_eq!(
            sent_payload.data,
            base64::prelude::BASE64_STANDARD.encode(binary_data)
        );
        assert_eq!(sent_payload.filename, Some("binary.dat".to_string()));

        // Verify we can decode back to original bytes
        let decoded = sent_payload.decode_bytes().unwrap();
        assert_eq!(decoded, binary_data);
    }

    #[tokio::test]
    async fn test_secret_client_large_binary_file() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Create a large binary file (1MB)
        let large_data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
        let payload = Payload::from_bytes(&large_data, Some("large_file.bin".to_string()));

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());

        // Verify the payload was handled correctly
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();

        // Verify filename
        assert_eq!(sent_payload.filename, Some("large_file.bin".to_string()));

        // Verify we can decode back to original bytes
        let decoded = sent_payload.decode_bytes().unwrap();
        assert_eq!(decoded.len(), large_data.len());
        assert_eq!(decoded, large_data);
    }

    #[tokio::test]
    async fn test_secret_client_empty_payload() {
        let mock_client = MockClient::new();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Create empty payload
        let payload = Payload::from_bytes(b"", None);

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_secret_client_special_filename_characters() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Test with special characters in filename
        let payload = Payload::from_bytes(
            b"test data",
            Some("file with spaces & special-chars!@#$%.txt".to_string()),
        );

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());

        // Verify the filename is preserved exactly
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();
        assert_eq!(
            sent_payload.filename,
            Some("file with spaces & special-chars!@#$%.txt".to_string())
        );
    }

    #[tokio::test]
    async fn test_secret_client_from_bytes_without_filename() {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Create payload from bytes without filename (text mode)
        let text_bytes = "Hello, this is text data".as_bytes();
        let payload = Payload::from_bytes(text_bytes, None);

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token)
            .await;

        assert!(result.is_ok());

        // Verify the payload
        let sent_data = mock_clone.get_sent_data().unwrap();
        let sent_payload: Payload = serde_json::from_str(&sent_data).unwrap();
        assert_eq!(sent_payload.filename, None);

        // Verify we can decode back to original text
        let decoded = sent_payload.decode_bytes().unwrap();
        assert_eq!(decoded, text_bytes);
        assert_eq!(
            String::from_utf8(decoded).unwrap(),
            "Hello, this is text data"
        );
    }
}
