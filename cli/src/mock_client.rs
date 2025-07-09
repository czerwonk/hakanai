#[cfg(test)]
pub mod test_utils {
    use std::sync::Arc;
    use std::time::Duration;
    use anyhow::Result;
    use async_trait::async_trait;
    use hakanai_lib::client::{Client, ClientError};
    use hakanai_lib::models::Payload;
    use hakanai_lib::observer::DataTransferObserver;
    use hakanai_lib::options::{SecretReceiveOptions, SecretSendOptions};
    use url::Url;
    use crate::factory::Factory;

    pub struct MockClient {
        send_should_fail: bool,
        send_error_message: Option<String>,
        send_result_url: Option<Url>,
        receive_should_fail: bool,
        receive_error_message: Option<String>,
        receive_result: Option<Payload>,
    }

    impl MockClient {
        pub fn new() -> Self {
            Self {
                send_should_fail: false,
                send_error_message: None,
                send_result_url: None,
                receive_should_fail: false,
                receive_error_message: None,
                receive_result: None,
            }
        }

        pub fn with_send_success(mut self, url: Url) -> Self {
            self.send_result_url = Some(url);
            self.send_should_fail = false;
            self
        }

        pub fn with_send_failure(mut self, error_message: String) -> Self {
            self.send_should_fail = true;
            self.send_error_message = Some(error_message);
            self
        }

        pub fn with_receive_success(mut self, payload: Payload) -> Self {
            self.receive_result = Some(payload);
            self.receive_should_fail = false;
            self
        }

        pub fn with_receive_failure(mut self, error_message: String) -> Self {
            self.receive_should_fail = true;
            self.receive_error_message = Some(error_message);
            self
        }
    }

    #[async_trait]
    impl Client<Payload> for MockClient {
        async fn send_secret(
            &self,
            _base_url: Url,
            _payload: Payload,
            _ttl: Duration,
            _token: String,
            _opts: Option<SecretSendOptions>,
        ) -> Result<Url, ClientError> {
            if self.send_should_fail {
                let error_msg = self
                    .send_error_message
                    .clone()
                    .unwrap_or_else(|| "Mock send error".to_string());
                Err(ClientError::Custom(error_msg))
            } else {
                Ok(self.send_result_url.clone().unwrap_or_else(|| {
                    Url::parse("https://example.com/s/mock123#mockkey").unwrap()
                }))
            }
        }

        async fn receive_secret(
            &self,
            _url: Url,
            _opts: Option<SecretReceiveOptions>,
        ) -> Result<Payload, ClientError> {
            if self.receive_should_fail {
                let error_msg = self
                    .receive_error_message
                    .clone()
                    .unwrap_or_else(|| "Mock receive error".to_string());
                Err(ClientError::Custom(error_msg))
            } else {
                Ok(self
                    .receive_result
                    .clone()
                    .unwrap_or_else(|| Payload::from_bytes(b"mock secret", None)))
            }
        }
    }

    /// Mock observer that does nothing (no console output during tests)
    pub struct MockObserver;

    #[async_trait]
    impl DataTransferObserver for MockObserver {
        async fn on_progress(&self, _bytes_transferred: u64, _total_bytes: u64) {
            // Do nothing - avoids console interference during tests
        }
    }

    /// Mock factory for dependency injection in tests
    pub struct MockFactory {
        client: MockClient,
    }

    impl MockFactory {
        pub fn new() -> Self {
            Self {
                client: MockClient::new(),
            }
        }

        pub fn with_client(mut self, client: MockClient) -> Self {
            self.client = client;
            self
        }
    }

    impl Factory for MockFactory {
        fn new_client(&self) -> impl Client<Payload> {
            MockClient {
                send_should_fail: self.client.send_should_fail,
                send_error_message: self.client.send_error_message.clone(),
                send_result_url: self.client.send_result_url.clone(),
                receive_should_fail: self.client.receive_should_fail,
                receive_error_message: self.client.receive_error_message.clone(),
                receive_result: self.client.receive_result.clone(),
            }
        }

        fn new_observer(&self, _label: &str) -> Result<Arc<dyn DataTransferObserver>> {
            Ok(Arc::new(MockObserver))
        }
    }
}
