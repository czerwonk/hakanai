// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
use std::sync::{Arc, Mutex};
#[cfg(any(test, feature = "testing"))]
use std::time::Duration;

#[cfg(any(test, feature = "testing"))]
use async_trait::async_trait;
#[cfg(any(test, feature = "testing"))]
use url::Url;

#[cfg(any(test, feature = "testing"))]
use crate::client::{Client, ClientError};
#[cfg(any(test, feature = "testing"))]
use crate::options::{SecretReceiveOptions, SecretSendOptions};

/// A generic mock client implementation using the builder pattern.
///
/// This mock client can be used to simulate both success and failure scenarios
/// for testing purposes. It supports any data type `T` that implements the required traits.
///
/// # Examples
///
/// ```
/// use hakanai_lib::client_mock::MockClient;
/// use url::Url;
///
/// // Create a mock client for Vec<u8>
/// let mock = MockClient::<Vec<u8>>::new()
///     .with_send_success(Url::parse("https://example.com/secret/123").expect("valid URL"))
///     .with_receive_success(b"test response".to_vec());
///
/// // Create a mock client that fails
/// let failing_mock = MockClient::<Vec<u8>>::new()
///     .with_send_failure("Network error".to_string())
///     .with_receive_failure("Not found".to_string());
/// ```
#[cfg(any(test, feature = "testing"))]
#[derive(Clone)]
pub struct MockClient<T> {
    // Data capture
    sent_data: Arc<Mutex<Option<T>>>,

    // Response configuration
    response_url: Option<Url>,
    response_data: Option<T>,

    // Error configuration
    send_should_fail: bool,
    send_error_message: Option<String>,
    receive_should_fail: bool,
    receive_error_message: Option<String>,
}

#[cfg(any(test, feature = "testing"))]
impl<T> MockClient<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Creates a new MockClient with default configuration.
    ///
    /// By default, the mock client will:
    /// - Return a default URL for send operations
    /// - Return default data for receive operations
    /// - Not fail unless explicitly configured to do so
    pub fn new() -> Self {
        Self {
            sent_data: Arc::new(Mutex::new(None)),
            response_url: Some(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
            ),
            response_data: None,
            send_should_fail: false,
            send_error_message: None,
            receive_should_fail: false,
            receive_error_message: None,
        }
    }

    /// Configure the mock to return a specific URL on successful send operations.
    pub fn with_send_success(mut self, url: Url) -> Self {
        self.response_url = Some(url);
        self.send_should_fail = false;
        self
    }

    /// Configure the mock to fail send operations with the given error message.
    pub fn with_send_failure(mut self, error_message: String) -> Self {
        self.send_should_fail = true;
        self.send_error_message = Some(error_message);
        self
    }

    /// Configure the mock to return specific data on successful receive operations.
    pub fn with_receive_success(mut self, data: T) -> Self {
        self.response_data = Some(data);
        self.receive_should_fail = false;
        self
    }

    /// Configure the mock to fail receive operations with the given error message.
    pub fn with_receive_failure(mut self, error_message: String) -> Self {
        self.receive_should_fail = true;
        self.receive_error_message = Some(error_message);
        self
    }

    /// Configure both the send URL and receive data for successful operations.
    pub fn with_success(mut self, url: Url, data: T) -> Self {
        self.response_url = Some(url);
        self.response_data = Some(data);
        self.send_should_fail = false;
        self.receive_should_fail = false;
        self
    }

    /// Configure the mock to fail all operations with the given error message.
    pub fn with_all_failures(mut self, error_message: String) -> Self {
        self.send_should_fail = true;
        self.send_error_message = Some(error_message.clone());
        self.receive_should_fail = true;
        self.receive_error_message = Some(error_message);
        self
    }

    /// Get the data that was sent to the mock client.
    ///
    /// This is useful for verifying that the correct data was passed to the client
    /// during testing.
    pub fn get_sent_data(&self) -> Option<T> {
        self.sent_data
            .lock()
            .expect("Unable to aquire lock")
            .clone()
    }

    /// Check if any data was sent to the mock client.
    pub fn was_send_called(&self) -> bool {
        self.sent_data
            .lock()
            .expect("Unable to aquire lock")
            .is_some()
    }

    /// Clear any captured sent data.
    pub fn clear_sent_data(&self) {
        *self.sent_data.lock().expect("Unable to aquire lock") = None;
    }
}

#[cfg(any(test, feature = "testing"))]
impl<T> Default for MockClient<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "testing"))]
#[async_trait]
impl<T> Client<T> for MockClient<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn send_secret(
        &self,
        _base_url: Url,
        data: T,
        _ttl: Duration,
        _token: String,
        _opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        // Capture the sent data
        *self.sent_data.lock().expect("Unable to aquire lock") = Some(data);

        // Return configured response
        if self.send_should_fail {
            let error_msg = self
                .send_error_message
                .clone()
                .unwrap_or_else(|| "Mock send error".to_string());
            Err(ClientError::Custom(error_msg))
        } else {
            Ok(self.response_url.clone().unwrap_or_else(|| {
                Url::parse("https://example.com/secret/default").expect("Failed to parse valid URL")
            }))
        }
    }

    async fn receive_secret(
        &self,
        _url: Url,
        _opts: Option<SecretReceiveOptions>,
    ) -> Result<T, ClientError> {
        if self.receive_should_fail {
            let error_msg = self
                .receive_error_message
                .clone()
                .unwrap_or_else(|| "Mock receive error".to_string());
            Err(ClientError::Custom(error_msg))
        } else {
            self.response_data
                .clone()
                .ok_or_else(|| ClientError::Custom("No response data configured".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Payload;
    use std::error::Error;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[tokio::test]
    async fn test_mock_client_vec_u8_send_success() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::new().with_send_success(
            Url::parse("https://test.com/secret/456").expect("Failed to parse valid URL"),
        );

        let test_data = b"test data".to_vec();
        let result = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                test_data.clone(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        let url = result?;
        assert_eq!(url.as_str(), "https://test.com/secret/456");
        assert_eq!(mock.get_sent_data(), Some(test_data));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_vec_u8_send_failure() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::new().with_send_failure("Network error".to_string());

        let test_data = b"test data".to_vec();
        let result = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                test_data.clone(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        assert!(result.is_err(), "Expected send error, got: {:?}", result);
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Network error"),
            _ => panic!("Expected Custom error"),
        }
        assert_eq!(mock.get_sent_data(), Some(test_data));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_vec_u8_receive_success() -> Result<()> {
        let response_data = b"response data".to_vec();
        let mock = MockClient::<Vec<u8>>::new().with_receive_success(response_data.clone());

        let result = mock
            .receive_secret(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
                None,
            )
            .await;

        let data = result?;
        assert_eq!(data, response_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_vec_u8_receive_failure() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::new().with_receive_failure("Not found".to_string());

        let result = mock
            .receive_secret(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
                None,
            )
            .await;

        assert!(result.is_err(), "Expected receive error, got: {:?}", result);
        match result.unwrap_err() {
            ClientError::Custom(msg) => assert_eq!(msg, "Not found"),
            _ => panic!("Expected Custom error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_payload_type() -> Result<()> {
        let payload = Payload::from_bytes(b"test payload").with_filename("test.txt");
        let mock = MockClient::<Payload>::new().with_receive_success(payload.clone());

        let result = mock
            .receive_secret(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
                None,
            )
            .await;

        let received = result?;
        assert_eq!(received.data, payload.data);
        assert_eq!(received.filename, payload.filename);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_builder_pattern() -> Result<()> {
        let test_url =
            Url::parse("https://test.com/secret/abc").expect("Failed to parse valid URL");
        let test_data = b"test response".to_vec();

        let mock = MockClient::new().with_success(test_url.clone(), test_data.clone());

        // Test send
        let send_result = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                b"send data".to_vec(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        let send_url = send_result?;
        assert_eq!(send_url, test_url);

        // Test receive
        let receive_result = mock
            .receive_secret(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
                None,
            )
            .await;

        let receive_data = receive_result?;
        assert_eq!(receive_data, test_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_all_failures() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::new().with_all_failures("Everything fails".to_string());

        // Test send failure
        let send_result = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                b"test".to_vec(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        assert!(
            send_result.is_err(),
            "Expected send failure, got: {:?}",
            send_result
        );

        // Test receive failure
        let receive_result = mock
            .receive_secret(
                Url::parse("https://example.com/secret/123").expect("Failed to parse valid URL"),
                None,
            )
            .await;

        assert!(
            receive_result.is_err(),
            "Expected receive failure, got: {:?}",
            receive_result
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_data_capture() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::new();

        assert!(!mock.was_send_called());
        assert_eq!(mock.get_sent_data(), None);

        let test_data = b"captured data".to_vec();
        let _ = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                test_data.clone(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        assert!(mock.was_send_called());
        assert_eq!(mock.get_sent_data(), Some(test_data));

        mock.clear_sent_data();
        assert!(!mock.was_send_called());
        assert_eq!(mock.get_sent_data(), None);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_client_default_impl() -> Result<()> {
        let mock = MockClient::<Vec<u8>>::default();

        let result = mock
            .send_secret(
                Url::parse("https://example.com").expect("Failed to parse valid URL"),
                b"test".to_vec(),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await;

        let url = result?;
        assert_eq!(url.as_str(), "https://example.com/secret/123");
        Ok(())
    }
}
