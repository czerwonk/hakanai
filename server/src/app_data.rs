// SPDX-License-Identifier: Apache-2.0

use std::time;

use crate::data_store::DataStore;
use crate::observer::ObserverManager;
use crate::token::{TokenCreator, TokenValidator};

#[derive(Clone, Debug)]
pub struct AnonymousOptions {
    pub allowed: bool,

    /// The maximum upload size allowed for anonymous users, in bytes.
    pub upload_size_limit: usize,
}

/// AppData stores the application's shared state.
pub struct AppData {
    /// The data store for persisting application data.
    pub data_store: Box<dyn DataStore>,

    /// The token validator for authentication.
    pub token_validator: Box<dyn TokenValidator>,

    /// The token creator for admin API.
    pub token_creator: Box<dyn TokenCreator>,

    /// The maximum time-to-live (TTL) for secrets
    pub max_ttl: time::Duration,

    /// Defines whether the application can be used without authentication and limits for anonymous users.
    pub anonymous_usage: AnonymousOptions,

    /// Pre-rendered impressum HTML page (built at startup if configured)
    pub impressum_html: Option<String>,

    /// Pre-rendered privacy policy HTML page (built at startup if configured)
    pub privacy_html: Option<String>,

    /// The observer manager for secret lifecycle events.
    pub observer_manager: ObserverManager,

    /// Whether to show the token input field in the web interface
    pub show_token_input: bool,

    /// IP ranges that bypass size limits
    pub trusted_ip_ranges: Option<Vec<ipnet::IpNet>>,

    /// HTTP header to check for client IP
    pub trusted_ip_header: String,

    /// HTTP header to check for client country (for geo-restrictions)
    pub country_header: Option<String>,

    /// HTTP header to check for client ASN (for geo-restrictions)
    pub asn_header: Option<String>,

    /// The maximum upload size allowed for the server, in bytes.
    pub upload_size_limit: usize,
}

#[cfg(test)]
impl Default for AppData {
    fn default() -> Self {
        use crate::test_utils::MockDataStore;
        use crate::token::MockTokenManager;

        Self {
            data_store: Box::new(MockDataStore::new()),
            token_validator: Box::new(MockTokenManager::new()),
            token_creator: Box::new(MockTokenManager::new()),
            max_ttl: time::Duration::from_secs(86400), // 24 hours
            anonymous_usage: AnonymousOptions {
                allowed: false,
                upload_size_limit: 32 * 1024, // 32KB
            },
            impressum_html: None,
            privacy_html: None,
            observer_manager: ObserverManager::new(),
            show_token_input: false,
            trusted_ip_ranges: None,
            trusted_ip_header: "x-forwarded-for".to_string(),
            country_header: None,
            asn_header: None,
            upload_size_limit: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl AppData {
    /// Builder pattern functions for testing
    #[cfg(test)]
    pub fn with_data_store(mut self, data_store: Box<dyn DataStore>) -> Self {
        self.data_store = data_store;
        self
    }

    #[cfg(test)]
    pub fn with_token_validator(mut self, token_validator: Box<dyn TokenValidator>) -> Self {
        self.token_validator = token_validator;
        self
    }

    #[cfg(test)]
    pub fn with_token_creator(mut self, token_creator: Box<dyn TokenCreator>) -> Self {
        self.token_creator = token_creator;
        self
    }

    #[cfg(test)]
    pub fn with_max_ttl(mut self, max_ttl: time::Duration) -> Self {
        self.max_ttl = max_ttl;
        self
    }

    #[cfg(test)]
    pub fn with_anonymous_usage(mut self, anonymous_usage: AnonymousOptions) -> Self {
        self.anonymous_usage = anonymous_usage;
        self
    }

    #[cfg(test)]
    pub fn with_impressum_html(mut self, impressum_html: &str) -> Self {
        self.impressum_html = Some(impressum_html.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_privacy_html(mut self, privacy_html: &str) -> Self {
        self.privacy_html = Some(privacy_html.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_trusted_ip_ranges(mut self, trusted_ip_ranges: Option<Vec<ipnet::IpNet>>) -> Self {
        self.trusted_ip_ranges = trusted_ip_ranges;
        self
    }

    #[cfg(test)]
    pub fn with_trusted_ip_header(mut self, trusted_ip_header: String) -> Self {
        self.trusted_ip_header = trusted_ip_header;
        self
    }

    #[cfg(test)]
    pub fn with_country_header(mut self, country_header: Option<String>) -> Self {
        self.country_header = country_header;
        self
    }

    #[cfg(test)]
    pub fn with_asn_header(mut self, asn_header: Option<String>) -> Self {
        self.asn_header = asn_header;
        self
    }
}
