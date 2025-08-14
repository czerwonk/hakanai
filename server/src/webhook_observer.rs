// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::time::Duration;

use actix_web::http::header::HeaderMap;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};

use crate::observer::SecretObserver;

const SAFE_HEADERS: [&str; 5] = [
    "user-agent",
    "x-forwarded-for",
    "x-forwarded-proto",
    "x-real-ip",
    "x-request-id",
];

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
    /// Filtered HTTP headers (safe subset).
    pub headers: HashMap<String, String>,
}

/// Sends webhook notifications for secret events.
pub struct WebhookObserver {
    url: String,
    auth_token: Option<String>,
    client: reqwest::Client,
}

#[async_trait]
impl SecretObserver for WebhookObserver {
    #[instrument(skip(self, headers))]
    async fn on_secret_created(&self, secret_id: uuid::Uuid, headers: HeaderMap) {
        let payload = WebhookPayload {
            secret_id,
            action: WebhookAction::Created,
            headers: filter_headers(&headers),
        };
        self.send_webhook(payload).await;
    }

    #[instrument(skip(self, headers))]
    async fn on_secret_retrieved(&self, secret_id: uuid::Uuid, headers: HeaderMap) {
        let payload = WebhookPayload {
            secret_id,
            action: WebhookAction::Retrieved,
            headers: filter_headers(&headers),
        };
        self.send_webhook(payload).await;
    }
}

impl WebhookObserver {
    /// Creates a new webhook observer.
    pub fn new(url: String, auth_token: Option<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(WebhookObserver {
            url,
            auth_token,
            client,
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
}

fn filter_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut filtered = HashMap::new();

    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if SAFE_HEADERS.contains(&key_str.as_str()) {
            if let Ok(value_str) = value.to_str() {
                filtered.insert(key_str, value_str.to_string());
            } else {
                warn!("Failed to convert header value to string for key: {}", key);
            }
        }
    }

    filtered
}
