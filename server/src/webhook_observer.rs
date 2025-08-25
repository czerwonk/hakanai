// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::time::Duration;

use actix_web::http::header::HeaderMap;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::observer::{SecretEventContext, SecretObserver};

/// Webhook action types.
#[derive(Serialize, Deserialize, Debug)]
pub enum WebhookAction {
    Created,
    Retrieved,
}

/// Webhook notification payload.
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookPayload {
    /// UUID of the secret.
    pub secret_id: uuid::Uuid,
    /// Action that triggered the webhook.
    pub action: WebhookAction,
    /// Additional details about the request (e.g. exctracted from headers)
    pub details: HashMap<String, String>,
}

/// Sends webhook notifications for secret events.
pub struct WebhookObserver {
    url: String,
    auth_token: Option<String>,
    client: reqwest::Client,
    header_names: Vec<String>,
}

#[async_trait]
impl SecretObserver for WebhookObserver {
    #[instrument(skip(self, context))]
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
        let mut details = self.filter_headers(&context.headers);
        if let Some(user_type) = &context.user_type {
            details.insert("user_type".to_string(), user_type.to_string());
        }

        if let Some(restrictions) = &context.restrictions {
            details.insert("restrictions".to_string(), restrictions.to_string());
        }

        let payload = WebhookPayload {
            secret_id,
            action: WebhookAction::Created,
            details,
        };
        self.send_webhook(payload).await;
    }

    #[instrument(skip(self, context))]
    async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext) {
        let payload = WebhookPayload {
            secret_id,
            action: WebhookAction::Retrieved,
            details: self.filter_headers(&context.headers),
        };
        self.send_webhook(payload).await;
    }
}

impl WebhookObserver {
    /// Creates a new webhook observer.
    pub fn new(url: String, auth_token: Option<String>, header_names: Vec<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(WebhookObserver {
            url,
            auth_token,
            client,
            header_names: header_names.iter().map(|h| h.to_lowercase()).collect(),
        })
    }

    #[instrument(skip(self))]
    async fn send_webhook(&self, payload: WebhookPayload) {
        let mut req = self.client.post(&self.url).json(&payload);

        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }

        tokio::spawn(async move {
            if let Err(e) = req.send().await {
                warn!("Webhook failed: {e}");
            }
        });
    }

    fn filter_headers(&self, headers: &HeaderMap) -> HashMap<String, String> {
        let mut filtered = HashMap::new();

        for (key, value) in headers.iter() {
            let key_str = key.as_str().to_lowercase();
            if self.header_names.contains(&key_str) {
                if let Ok(value_str) = value.to_str() {
                    filtered.insert(key_str, value_str.to_string());
                } else {
                    warn!("Failed to convert header value to string for key: {}", key);
                }
            }
        }

        filtered
    }
}
