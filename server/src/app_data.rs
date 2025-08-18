// SPDX-License-Identifier: Apache-2.0

use std::time;

use crate::data_store::DataStore;
use crate::observer::ObserverManager;
use crate::token::{TokenCreator, TokenValidator};

#[derive(Clone, Debug)]
pub struct AnonymousOptions {
    pub allowed: bool,

    /// The maximum size of uploads allowed for anonymous users, in bytes.
    pub upload_size_limit: u64,
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
}
