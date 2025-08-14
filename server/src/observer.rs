// SPDX-License-Identifier: MIT

use actix_web::http::header::HeaderMap;
use async_trait::async_trait;
use tracing::instrument;

/// Observer for secret lifecycle events.
#[async_trait]
pub trait SecretObserver: Send + Sync {
    /// Called when a secret is created.
    async fn on_secret_created(&self, secret_id: uuid::Uuid, headers: HeaderMap);

    /// Called when a secret is retrieved.
    async fn on_secret_retrieved(&self, secret_id: uuid::Uuid, headers: HeaderMap);
}

pub struct ObserverManager {
    observers: Vec<Box<dyn SecretObserver>>,
}

impl ObserverManager {
    pub fn new() -> Self {
        ObserverManager {
            observers: Vec::new(),
        }
    }

    pub fn register_observer(&mut self, observer: Box<dyn SecretObserver>) {
        self.observers.push(observer);
    }

    /// Notify observers when a secret is created.
    #[instrument(skip(self, headers))]
    pub async fn notify_secret_created(&self, secret_id: uuid::Uuid, headers: HeaderMap) {
        for observer in &self.observers {
            observer.on_secret_created(secret_id, headers.clone()).await;
        }
    }

    /// Notify observers when a secret is retrieved.
    #[instrument(skip(self, headers))]
    pub async fn notify_secret_retrieved(&self, secret_id: uuid::Uuid, headers: HeaderMap) {
        for observer in &self.observers {
            observer
                .on_secret_retrieved(secret_id, headers.clone())
                .await;
        }
    }
}
