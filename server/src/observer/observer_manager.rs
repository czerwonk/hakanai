// SPDX-License-Identifier: Apache-2.0

use tracing::instrument;
use uuid::Uuid;

use super::{SecretEventContext, SecretObserver};

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
    #[instrument(skip(self, context))]
    pub async fn notify_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
        for observer in &self.observers {
            observer.on_secret_created(secret_id, context).await;
        }
    }

    /// Notify observers when a secret is retrieved.
    #[instrument(skip(self, context))]
    pub async fn notify_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext) {
        for observer in &self.observers {
            observer.on_secret_retrieved(secret_id, context).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::mock_observer::MockObserver;
    use actix_web::http::header::HeaderMap;

    #[tokio::test]
    async fn test_notify_secret_created_with_multiple_observers() {
        let mut manager = ObserverManager::new();
        let observer1 = MockObserver::new();
        let observer1_ref = observer1.clone();
        let observer2 = MockObserver::new();
        let observer2_ref = observer2.clone();

        manager.register_observer(Box::new(observer1));
        manager.register_observer(Box::new(observer2));

        let secret_id = Uuid::new_v4();
        let context = SecretEventContext::new(HeaderMap::new());

        manager.notify_secret_created(secret_id, &context).await;

        let created_events_1 = observer1_ref.get_created_events();
        let created_events_2 = observer2_ref.get_created_events();

        assert_eq!(
            created_events_1.len(),
            1,
            "First observer should be called for secret creation"
        );
        assert_eq!(
            created_events_1[0].0, secret_id,
            "First observer should receive correct secret ID"
        );
        assert_eq!(
            created_events_2.len(),
            1,
            "Second observer should be called for secret creation"
        );
        assert_eq!(
            created_events_2[0].0, secret_id,
            "Second observer should receive correct secret ID"
        );
    }

    #[tokio::test]
    async fn test_notify_secret_retrieved_with_multiple_observers() {
        let mut manager = ObserverManager::new();
        let observer1 = MockObserver::new();
        let observer1_ref = observer1.clone();
        let observer2 = MockObserver::new();
        let observer2_ref = observer2.clone();

        manager.register_observer(Box::new(observer1));
        manager.register_observer(Box::new(observer2));

        let secret_id = Uuid::new_v4();
        let context = SecretEventContext::new(HeaderMap::new());

        manager.notify_secret_retrieved(secret_id, &context).await;

        let retrieved_events_1 = observer1_ref.get_retrieved_events();
        let retrieved_events_2 = observer2_ref.get_retrieved_events();

        assert_eq!(
            retrieved_events_1.len(),
            1,
            "First observer should be called for secret retrieval"
        );
        assert_eq!(
            retrieved_events_1[0].0, secret_id,
            "First observer should receive correct secret ID"
        );
        assert_eq!(
            retrieved_events_2.len(),
            1,
            "Second observer should be called for secret retrieval"
        );
        assert_eq!(
            retrieved_events_2[0].0, secret_id,
            "Second observer should receive correct secret ID"
        );
    }
}
