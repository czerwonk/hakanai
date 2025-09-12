// SPDX-License-Identifier: Apache-2.0

mod observer_manager;
mod secret_event_context;
mod webhook_observer;

#[cfg(test)]
mod mock_observer;

pub use observer_manager::ObserverManager;
pub use secret_event_context::SecretEventContext;
pub use webhook_observer::WebhookObserver;

#[cfg(test)]
pub use mock_observer::MockObserver;

use async_trait::async_trait;
use uuid::Uuid;

/// Observer for secret lifecycle events.
#[async_trait]
pub trait SecretObserver: Send + Sync {
    /// Called when a secret is created.
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext);

    /// Called when a secret is retrieved.
    async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext);
}
