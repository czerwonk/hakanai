// SPDX-License-Identifier: Apache-2.0

use actix_web::http::header::HeaderMap;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;

use crate::user::UserType;

#[derive(Clone)]
pub struct SecretEventContext {
    /// Headers associated with the secret event.
    pub headers: HeaderMap,
    /// User type associated with the secret event, if any.
    pub user_type: Option<UserType>,
    /// Restrictions associated with the secret event, if any.
    pub restrictions: Option<SecretRestrictions>,
}

impl SecretEventContext {
    pub fn new(headers: HeaderMap) -> Self {
        SecretEventContext {
            headers,
            user_type: None,
            restrictions: None,
        }
    }

    pub fn with_user_type(mut self, user_type: UserType) -> Self {
        self.user_type = Some(user_type);
        self
    }

    pub fn with_restrictions(mut self, restrictions: SecretRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
    }
}

/// Observer for secret lifecycle events.
#[async_trait]
pub trait SecretObserver: Send + Sync {
    /// Called when a secret is created.
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext);

    /// Called when a secret is retrieved.
    async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext);
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
