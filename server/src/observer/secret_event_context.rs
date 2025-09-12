// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use actix_web::http::header::HeaderMap;

use hakanai_lib::models::SecretRestrictions;

use crate::user_type::UserType;

/// Context for secret events, providing additional metadata.
#[derive(Clone)]
pub struct SecretEventContext {
    /// Time to live (TTL) of the secret.
    pub ttl: Option<Duration>,
    /// Headers associated with the secret event.
    pub headers: HeaderMap,
    /// User type associated with the secret event, if any.
    pub user_type: Option<UserType>,
    /// Restrictions associated with the secret event, if any.
    pub restrictions: Option<SecretRestrictions>,
    /// Size of the secret, if known.
    pub size: Option<usize>,
}

impl SecretEventContext {
    pub fn new(headers: HeaderMap) -> Self {
        SecretEventContext {
            headers,
            user_type: None,
            restrictions: None,
            ttl: None,
            size: None,
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

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }
}
