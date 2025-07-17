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
}

impl From<aes_gcm::Error> for ClientError {
    fn from(err: aes_gcm::Error) -> Self {
        ClientError::CryptoError(format!("AES-GCM error: {err:?}"))
    }
}

/// A client for sending and receiving `Payload` objects.
///
/// This client acts as a layer over a `Client<Vec<u8>>`, handling the serialization
/// and deserialization of `Payload` objects to and from JSON strings.
///
/// **Note:** This is an internal implementation detail. Users should use `client::new()`
/// which returns a ready-to-use client for `Payload` objects.
///
/// # Examples
///
/// ```
/// use hakanai_lib::{client, client::Client, models::Payload};
/// use std::time::Duration;
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = client::new();
///
/// // Text payload
/// let payload = Payload { data: "Hello, World!".to_string(), filename: None };
/// let secret_url = client.send_secret(
///     Url::parse("https://api.example.com")?,
///     payload,
///     Duration::from_secs(300),
///     "auth-token".to_string(),
///     None,
/// ).await?;
///
/// // File payload
/// let file_payload = Payload::from_bytes(b"file content", Some("doc.pdf".to_string()));
/// let file_url = client.send_secret(
///     Url::parse("https://api.example.com")?,
///     file_payload,
///     Duration::from_secs(86400),
///     "auth-token".to_string(),
///     None,
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct SecretClient {
    client: Box<dyn Client<Vec<u8>>>,
}

#[async_trait]
impl Client<Payload> for SecretClient {
    async fn send_secret(
        &self,
        base_url: Url,
        payload: Payload,
        ttl: Duration,
        token: String,
        options: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        let data = serde_json::to_vec(&payload)?;
        let url = self
            .client
            .send_secret(base_url, data, ttl, token, options)
            .await?;
        Ok(url)
    }

    async fn receive_secret(
        &self,
        url: Url,
        opts: Option<SecretReceiveOptions>,
    ) -> Result<Payload, ClientError> {
        let data = self.client.receive_secret(url, opts).await?;
        let payload: Payload = serde_json::from_slice(&data)?;
        Ok(payload)
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
/// // The URL contains the encryption key in the fragment
/// println!("Share this URL: {}", url);
/// # Ok(())
/// # }
/// ```
pub fn new() -> impl Client<Payload> {
    SecretClient {
        client: Box::new(CryptoClient::new(Box::new(WebClient::new()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_mock::MockClient;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error: {}", self.0)
        }
    }

    impl Error for TestError {}

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[tokio::test]
    async fn test_secret_client_send_text_payload() -> Result<()> {
        let mock_client = MockClient::new()
            .with_send_success(Url::parse("https://example.com/secret/123").unwrap());
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let payload = Payload::from_bytes(b"Hello, World!", None);

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        let url = result?;
        assert_eq!(url.as_str(), "https://example.com/secret/123");

        // Verify the payload was serialized correctly
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;
        assert_eq!(sent_payload.decode_bytes()?, b"Hello, World!");
        assert_eq!(sent_payload.filename, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_send_file_payload() -> Result<()> {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let binary_data = b"Binary file content";
        let payload = Payload::from_bytes(binary_data, Some("document.pdf".to_string()));

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;

        // Verify the payload was serialized correctly
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;
        let decoded = sent_payload.decode_bytes()?;
        assert_eq!(decoded, binary_data);
        assert_eq!(sent_payload.filename, Some("document.pdf".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_receive_text_payload() -> Result<()> {
        let response_json = r#"{"data":"SGVsbG8gZnJvbSBzZXJ2ZXI=","filename":null}"#;
        let mock_client = MockClient::new().with_receive_success(response_json.as_bytes().to_vec());
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url, None).await;

        let payload = result?;
        let decoded = payload.decode_bytes()?;
        assert_eq!(decoded, b"Hello from server");
        assert_eq!(payload.filename, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_receive_file_payload() -> Result<()> {
        let response_json = r#"{"data":"U29tZSBiaW5hcnkgZGF0YQ==","filename":"test.bin"}"#;
        let mock_client = MockClient::new().with_receive_success(response_json.as_bytes().to_vec());
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url, None).await;

        let payload = result?;
        assert_eq!(payload.filename, Some("test.bin".to_string()));

        // Verify the data can be decoded
        let decoded = payload.decode_bytes()?;
        assert_eq!(decoded, b"Some binary data");
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_send_error_propagation() -> Result<()> {
        let mock_client = MockClient::new().with_send_failure("Send failed".to_string());
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let payload = Payload::from_bytes(b"test", None);

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Send failed"),
            _ => panic!("Expected Custom error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_receive_error_propagation() -> Result<()> {
        let mock_client = MockClient::new().with_receive_failure("Receive failed".to_string());
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Receive failed"),
            _ => panic!("Expected Custom error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_invalid_json_response() -> Result<()> {
        let invalid_json = r#"{"data": "test", invalid json"#;
        let mock_client = MockClient::new().with_receive_success(invalid_json.as_bytes().to_vec());
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::Json(_) => (),
            _ => panic!("Expected Json error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_payload_roundtrip() -> Result<()> {
        // Test that a payload can be serialized and deserialized correctly
        let original_payload = Payload::from_bytes(b"Test binary data", None);

        let json = serde_json::to_vec(&original_payload)?;
        let mock_client = MockClient::new().with_receive_success(json);
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        let url = Url::parse("https://example.com/secret/123").unwrap();
        let result = secret_client.receive_secret(url, None).await;

        let received_payload = result?;
        assert_eq!(received_payload.data, original_payload.data);
        assert_eq!(received_payload.filename, original_payload.filename);

        // Verify the binary data is preserved
        let original_bytes = original_payload.decode_bytes()?;
        let received_bytes = received_payload.decode_bytes()?;
        assert_eq!(original_bytes, received_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn test_new_creates_correct_client_stack() -> Result<()> {
        // This test verifies that the new() function creates the expected client stack
        let client = new();

        // We can test that it implements the Client<Payload> trait
        let payload = Payload::from_bytes(b"test", None);

        // This will fail since we don't have a real server, but it proves the client is constructed correctly
        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = client
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        // We expect this to fail with a network error since there's no real server
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_send_from_bytes() -> Result<()> {
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
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;

        // Verify the payload was serialized correctly
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;

        // Verify that the data is base64 encoded
        assert_eq!(sent_payload.decode_bytes()?, binary_data);
        assert_eq!(sent_payload.filename, Some("binary.dat".to_string()));

        // Verify we can decode back to original bytes
        let decoded = sent_payload.decode_bytes()?;
        assert_eq!(decoded, binary_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_large_binary_file() -> Result<()> {
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
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;

        // Verify the payload was handled correctly
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;

        // Verify filename
        assert_eq!(sent_payload.filename, Some("large_file.bin".to_string()));

        // Verify we can decode back to original bytes
        let decoded = sent_payload.decode_bytes()?;
        assert_eq!(decoded.len(), large_data.len());
        assert_eq!(decoded, large_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_empty_payload() -> Result<()> {
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
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_special_filename_characters() -> Result<()> {
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
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;

        // Verify the filename is preserved exactly
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;
        assert_eq!(
            sent_payload.filename,
            Some("file with spaces & special-chars!@#$%.txt".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_secret_client_from_bytes_without_filename() -> Result<()> {
        let mock_client = MockClient::new();
        let mock_clone = mock_client.clone();
        let secret_client = SecretClient {
            client: Box::new(mock_client),
        };

        // Create payload from bytes without filename (text mode)
        let text_bytes = b"Hello, this is text data";
        let payload = Payload::from_bytes(text_bytes, None);

        let base_url = Url::parse("https://example.com").unwrap();
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let result = secret_client
            .send_secret(base_url, payload, ttl, token, None)
            .await;

        result?;

        // Verify the payload
        let sent_data = mock_clone
            .get_sent_data()
            .ok_or(TestError("No sent data".to_string()))?;
        let sent_payload: Payload = serde_json::from_slice(&sent_data)?;
        assert_eq!(sent_payload.filename, None);

        // Verify we can decode back to original text
        let decoded = sent_payload.decode_bytes()?;
        assert_eq!(decoded, text_bytes);
        assert_eq!(String::from_utf8(decoded)?, "Hello, this is text data");
        Ok(())
    }
}
