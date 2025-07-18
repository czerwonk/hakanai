use std::time::Duration;

use clap::Parser;

/// Represents the command-line arguments for the server.
#[derive(Parser)]
#[command(
    version,
    name = "hakanai-server",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist."
)]
pub struct Args {
    /// The port on which the server will listen for incoming connections.
    #[arg(
        short,
        long,
        value_name = "PORT",
        env = "HAKANAI_PORT",
        default_value = "8080"
    )]
    pub port: u16,

    /// The network address on which the server will listen.
    #[arg(
        short,
        long,
        value_name = "LISTEN_ADDRESS",
        env = "HAKANAI_LISTEN_ADDRESS",
        default_value = "127.0.0.1"
    )]
    pub listen_address: String,

    /// The Data Source Name (DSN) for the Redis database.
    #[arg(
        short,
        long,
        value_name = "REDIS_DSN",
        env = "HAKANAI_REDIS_DSN",
        default_value = "redis://127.0.0.1:6379/"
    )]
    pub redis_dsn: String,

    #[arg(
        short,
        long,
        value_name = "UPLOAD_SIZE_LIMIT",
        env = "HAKANAI_UPLOAD_SIZE_LIMIT",
        default_value = "10240",
        help = "Upload size limit in kilobytes. Defaults to 10 MB."
    )]
    pub upload_size_limit: u64,

    #[arg(
        short,
        long,
        value_name = "CORS_ALLOWED_ORIGINS",
        env = "HAKANAI_CORS_ALLOWED_ORIGINS",
        help = "Allowed origins for CORS requests. If not set, CORS is disabled."
    )]
    pub cors_allowed_origins: Option<Vec<String>>,

    #[arg(
        long,
        default_value = "7d",
        env = "HAKANAI_MAX_TTL",
        help = "Maximum allowed TTL for secrets.",
        value_parser = humantime::parse_duration,
    )]
    pub max_ttl: Duration,

    #[arg(
        long,
        default_value = "false",
        env = "HAKANAI_ALLOW_ANONYMOUS",
        help = "Allow anonymous access to the server. If set, users can create and view secrets without authentication."
    )]
    pub allow_anonymous: bool,

    #[arg(
        long,
        default_value = "32",
        env = "HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT",
        help = "Upload size limit for anonymous users in kilobytes. Defaults to 32KB"
    )]
    pub anonymous_upload_size_limit: u64,
}
