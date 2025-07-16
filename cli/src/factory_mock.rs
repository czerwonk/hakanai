#[cfg(test)]
pub mod test_utils {
    use crate::factory::Factory;
    use anyhow::Result;
    use async_trait::async_trait;
    use hakanai_lib::client::Client;
    use hakanai_lib::client_mock::MockClient;
    use hakanai_lib::models::Payload;
    use hakanai_lib::observer::DataTransferObserver;
    use std::sync::Arc;

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
        client: MockClient<Payload>,
    }

    impl MockFactory {
        pub fn new() -> Self {
            Self {
                client: MockClient::new(),
            }
        }

        pub fn with_client(mut self, client: MockClient<Payload>) -> Self {
            self.client = client;
            self
        }
    }

    impl Factory for MockFactory {
        fn new_client(&self) -> impl Client<Payload> {
            self.client.clone()
        }

        fn new_observer(&self, _label: &str) -> Result<Arc<dyn DataTransferObserver>> {
            Ok(Arc::new(MockObserver))
        }
    }
}
