// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::restrictions::SecretRestrictions;

/// Represents the request to create a new secret.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostSecretRequest {
    /// The secret data to be stored.
    pub data: String,

    /// The duration until the secret expires.
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub expires_in: Duration,

    /// Access restrictions for the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<SecretRestrictions>,
}

impl PostSecretRequest {
    /// Creates a new `PostSecretRequest`.
    ///
    /// # Arguments
    ///
    /// * `data` - The secret data.
    /// * `expires_in` - The duration until expiration.
    pub fn new(data: String, expires_in: Duration) -> Self {
        Self {
            data,
            expires_in,
            restrictions: None,
        }
    }

    /// Sets access restrictions for the secret
    pub fn with_restrictions(mut self, restrictions: SecretRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
    }
}

/// Represents the response after creating a new secret.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostSecretResponse {
    /// The unique identifier of the created secret.
    pub id: uuid::Uuid,
}

impl PostSecretResponse {
    /// Creates a new `PostSecretResponse`.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the secret.
    pub fn new(id: uuid::Uuid) -> Self {
        Self { id }
    }
}
