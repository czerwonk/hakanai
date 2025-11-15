// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use clap::Parser;
use url::Url;

use hakanai_lib::utils::human_size;

/// Represents the arguments for the `token` command.
#[derive(Debug, Clone, Parser)]
pub struct TokenArgs {
    #[arg(
        short,
        long,
        default_value = "http://localhost:8080",
        env = "HAKANAI_SERVER",
        help = "Hakanai Server URL to request the token from (eg. https://hakanai.link)."
    )]
    pub server: Url,

    #[arg(
        long,
        default_value = "30d",
        env = "HAKANAI_TOKEN_TTL",
        help = "Time until the token expires.",
        value_parser = humantime::parse_duration,
    )]
    pub ttl: Duration,

    #[arg(
        short,
        long,
        help = "Optional upload size limit for secret data before encryption (e.g., 1m, 500k, 1024).",
        value_parser = human_size::parse,
    )]
    pub limit: Option<i64>,

    #[arg(
        long,
        help = "If set, the token can only be used once.",
        default_value_t = false
    )]
    pub one_time: bool,
}
