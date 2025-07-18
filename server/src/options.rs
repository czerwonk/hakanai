use std::time::Duration;

use clap::Parser;
use hakanai_lib::utils::size_parser::parse_size_limit;

/// Parse a size limit for server configuration, returns value in bytes
fn parse_size_limit_bytes(s: &str) -> Result<u64, String> {
    let bytes = parse_size_limit(s)?;
    Ok(bytes.max(1) as u64)
}

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
        default_value = "10m",
        help = "Upload size limit (e.g., 10m, 1024k, 5242880). Defaults to 10 MB.",
        value_parser = parse_size_limit_bytes
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
        default_value = "32k",
        env = "HAKANAI_ANONYMOUS_UPLOAD_SIZE_LIMIT",
        help = "Upload size limit for anonymous users (e.g., 32k, 1m, 2048). Defaults to 32KB.",
        value_parser = parse_size_limit_bytes
    )]
    pub anonymous_upload_size_limit: u64,

    #[arg(
        long,
        default_value = "false",
        env = "HAKANAI_ENABLE_ADMIN_TOKEN",
        help = "Enable admin token and token management API"
    )]
    pub enable_admin_token: bool,

    #[arg(
        long,
        default_value = "false",
        help = "Force regenerate admin token (overwrites existing). Requires --enable-admin-token."
    )]
    pub reset_admin_token: bool,

    #[arg(
        long,
        default_value = "false",
        help = "Clear all user tokens and regenerate a new default token."
    )]
    pub reset_user_tokens: bool,
}

impl Args {
    /// Validates configuration parameters for compatibility and logical consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.reset_admin_token && !self.enable_admin_token {
            return Err("--reset-admin-token requires --enable-admin-token".to_string());
        }

        if self.anonymous_upload_size_limit > self.upload_size_limit {
            return Err(
                "--anonymous-size-limit cannot be larger than --upload-size-limit".to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args() -> Args {
        Args {
            port: 8080,
            listen_address: "127.0.0.1".to_string(),
            redis_dsn: "redis://127.0.0.1:6379/".to_string(),
            upload_size_limit: 10 * 1024 * 1024, // 10MB in bytes
            cors_allowed_origins: None,
            max_ttl: Duration::from_secs(604800),
            allow_anonymous: false,
            anonymous_upload_size_limit: 32 * 1024, // 32KB in bytes
            enable_admin_token: false,
            reset_admin_token: false,
            reset_user_tokens: false,
        }
    }

    #[test]
    fn test_validate_reset_admin_token_without_enable() {
        let args = Args {
            reset_admin_token: true,
            enable_admin_token: false,
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("--reset-admin-token requires --enable-admin-token")
        );
    }

    #[test]
    fn test_validate_reset_admin_token_with_enable() {
        let args = Args {
            reset_admin_token: true,
            enable_admin_token: true,
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_anonymous_size_limit_too_large() {
        let args = Args {
            anonymous_upload_size_limit: 100 * 1024, // 100KB in bytes
            upload_size_limit: 50 * 1024,            // 50KB in bytes
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("--anonymous-size-limit cannot be larger than --upload-size-limit")
        );
    }

    #[test]
    fn test_validate_valid_size_limits() {
        let args = Args {
            anonymous_upload_size_limit: 32 * 1024, // 32KB in bytes
            upload_size_limit: 10 * 1024 * 1024,    // 10MB in bytes
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_valid() {
        let args = Args {
            enable_admin_token: true,
            reset_admin_token: true,
            anonymous_upload_size_limit: 32 * 1024, // 32KB in bytes
            upload_size_limit: 10 * 1024 * 1024,    // 10MB in bytes
            allow_anonymous: true,
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edge_case_equal_limits() {
        let args = Args {
            anonymous_upload_size_limit: 1024 * 1024, // 1MB in bytes
            upload_size_limit: 1024 * 1024,           // 1MB in bytes
            ..create_test_args()
        };

        let result = args.validate();
        assert!(result.is_ok());
    }
}
