use std::sync::{Arc, Mutex};

use actix_web::http::header::HeaderMap;
use async_trait::async_trait;
use uuid::Uuid;

use super::{SecretEventContext, SecretObserver};

/// Mock implementation of SecretObserver trait for testing.
///
/// This mock allows tracking secret creation and retrieval events
/// for verification in tests.
#[derive(Clone)]
pub struct MockObserver {
    created_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
    retrieved_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
}

impl MockObserver {
    pub fn new() -> Self {
        MockObserver {
            created_events: Arc::new(Mutex::new(Vec::new())),
            retrieved_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_created_events(&self) -> Vec<(Uuid, HeaderMap)> {
        self.created_events.lock().unwrap().clone()
    }

    pub fn get_retrieved_events(&self) -> Vec<(Uuid, HeaderMap)> {
        self.retrieved_events.lock().unwrap().clone()
    }
}

#[async_trait]
impl SecretObserver for MockObserver {
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
        self.created_events
            .lock()
            .unwrap()
            .push((secret_id, context.headers.clone()));
    }

    async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext) {
        self.retrieved_events
            .lock()
            .unwrap()
            .push((secret_id, context.headers.clone()));
    }
}
