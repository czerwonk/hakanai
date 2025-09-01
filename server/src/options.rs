// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use clap::Parser;

use hakanai_lib::utils::{human_size, ip};

/// Parse a size limit for server configuration, returns value in bytes
fn parse_size_limit_bytes(s: &str) -> Result<usize, String> {
    let bytes = human_size::parse(s)?;
    Ok(bytes.max(1) as usize)
}

/// Represents the command-line arguments for the server.
#[derive(Parser, Clone)]
#[command(
    version,
    name = "hakanai-server",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist.",
    after_help = "LICENSE:\n  Licensed under the Apache License, Version 2.0\n  <https://www.apache.org/licenses/LICENSE-2.0>\n\nSOURCE:\n  <https://github.com/czerwonk/hakanai>"
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
        help = "Upload size limit for secret data before encryption (e.g., 10m, 1024k, 5242880). Defaults to 10 MB.",
        value_parser = parse_size_limit_bytes
    )]
    pub upload_size_limit: usize,

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
        help = "Upload size limit for anonymous users' secret data before encryption (e.g., 32k, 1m, 2048). Defaults to 32KB.",
        value_parser = parse_size_limit_bytes
    )]
    pub anonymous_upload_size_limit: usize,

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
        help = "Force regenerate admin token (overwrites existing) without starting the server."
    )]
    pub reset_admin_token: bool,

    #[arg(
        long,
        default_value = "false",
        help = "Only clears all user tokens and regenerate a new default token, does not start the server"
    )]
    pub reset_user_tokens: bool,

    /// Path to impressum text file for legal compliance
    #[arg(
        long,
        env = "HAKANAI_IMPRESSUM_FILE",
        help = "Path to impressum/legal information text file. When provided, an impressum link appears in the footer."
    )]
    pub impressum_file: Option<String>,

    /// Path to privacy policy text file for data protection compliance
    #[arg(
        long,
        env = "HAKANAI_PRIVACY_FILE",
        help = "Path to privacy policy/data protection text file. When provided, a privacy policy link appears in the footer."
    )]
    pub privacy_file: Option<String>,

    #[arg(long, env = "HAKANAI_WEBHOOK_URL", help = "Enable webhook")]
    pub webhook_url: Option<String>,

    #[arg(
        long,
        env = "HAKANAI_WEBHOOK_TOKEN",
        help = "Bearer token for webhook authentication"
    )]
    pub webhook_token: Option<String>,

    #[arg(
        env = "HAKANAI_WEBHOOK_HEADERS",
        value_delimiter = ',',
        default_value = "user-agent,x-forwarded-for,x-forwarded-proto,x-real-ip,x-request-id",
        help = "Comma-separated list of HTTP headers to include in webhook requests"
    )]
    pub webhook_headers: Vec<String>,

    #[arg(
        long,
        default_value = "false",
        env = "HAKANAI_SHOW_TOKEN_INPUT",
        help = "Show authentication token input field in web interface. If anonyomous access is enabled, this input is always shown."
    )]
    pub show_token_input: bool,

    #[arg(
        long,
        value_delimiter = ',',
        env = "HAKANAI_TRUSTED_IP_RANGES",
        help = "IP ranges (CIDR notation) that bypass size limits. Example: 10.0.0.0/8,192.168.1.0/24",
        value_parser = ip::parse_ipnet
    )]
    pub trusted_ip_ranges: Option<Vec<ipnet::IpNet>>,

    #[arg(
        long,
        default_value = "x-forwarded-for",
        env = "HAKANAI_TRUSTED_IP_HEADER",
        help = "HTTP header to check for client IP when behind a proxy. Common values: x-forwarded-for, x-real-ip, cf-connecting-ip"
    )]
    pub trusted_ip_header: String,

    #[arg(
        long,
        env = "HAKANAI_COUNTRY_HEADER",
        help = "HTTP header to check for client country code. This value is required to support Geo-IP based restrcttions"
    )]
    pub country_header: Option<String>,

    #[arg(
        long,
        env = "HAKANAI_ASN_HEADER",
        help = "HTTP header to check for client ASN. This value is required to support Geo-IP based restrcttions"
    )]
    pub asn_header: Option<String>,
}

impl Args {
    /// Validates configuration parameters for compatibility and logical consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.anonymous_upload_size_limit > self.upload_size_limit {
            return Err(
                "--anonymous-size-limit cannot be larger than --upload-size-limit".to_string(),
            );
        }

        if self.enable_admin_token && self.trusted_ip_ranges.is_none() {
            return Err("--enable-admin-token requires --trusted-ip-ranges to be set".to_string());
        }

        Ok(())
    }

    /// Loads impressum content from file if configured
    pub fn load_impressum_content(&self) -> std::io::Result<Option<String>> {
        match &self.impressum_file {
            Some(path) => std::fs::read_to_string(path).map(Some),
            None => Ok(None),
        }
    }

    /// Loads privacy policy content from file if configured
    pub fn load_privacy_content(&self) -> std::io::Result<Option<String>> {
        match &self.privacy_file {
            Some(path) => std::fs::read_to_string(path).map(Some),
            None => Ok(None),
        }
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
            impressum_file: None,
            privacy_file: None,
            webhook_url: None,
            webhook_token: None,
            webhook_headers: vec![],
            show_token_input: false,
            trusted_ip_ranges: None,
            trusted_ip_header: "x-forwarded-for".to_string(),
            country_header: None,
            asn_header: None,
        }
    }

    #[test]
    fn test_validate_reset_admin_token_with_enable() -> Result<(), String> {
        let args = Args {
            reset_admin_token: true,
            enable_admin_token: true,
            trusted_ip_ranges: Some(vec!["127.0.0.0/8".parse().unwrap()]),
            ..create_test_args()
        };

        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_anonymous_size_limit_too_large() {
        let args = Args {
            anonymous_upload_size_limit: 100 * 1024, // 100KB in bytes
            upload_size_limit: 50 * 1024,            // 50KB in bytes
            ..create_test_args()
        };

        let result = args.validate();
        assert!(
            result.is_err(),
            "Expected validation error, got: {:?}",
            result
        );
        assert!(
            result
                .unwrap_err()
                .contains("--anonymous-size-limit cannot be larger than --upload-size-limit")
        );
    }

    #[test]
    fn test_validate_valid_size_limits() -> Result<(), String> {
        let args = Args {
            anonymous_upload_size_limit: 32 * 1024, // 32KB in bytes
            upload_size_limit: 10 * 1024 * 1024,    // 10MB in bytes
            ..create_test_args()
        };

        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_all_valid() -> Result<(), String> {
        let args = Args {
            enable_admin_token: true,
            reset_admin_token: true,
            anonymous_upload_size_limit: 32 * 1024, // 32KB in bytes
            upload_size_limit: 10 * 1024 * 1024,    // 10MB in bytes
            allow_anonymous: true,
            trusted_ip_ranges: Some(vec!["127.0.0.0/8".parse().unwrap()]),
            ..create_test_args()
        };

        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_edge_case_equal_limits() -> Result<(), String> {
        let args = Args {
            anonymous_upload_size_limit: 1024 * 1024, // 1MB in bytes
            upload_size_limit: 1024 * 1024,           // 1MB in bytes
            ..create_test_args()
        };

        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_load_impressum_content_none() {
        let args = Args {
            impressum_file: None,
            ..create_test_args()
        };

        let result = args.load_impressum_content().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_impressum_content_nonexistent_file() {
        let args = Args {
            impressum_file: Some("/nonexistent/path/to/impressum.txt".to_string()),
            ..create_test_args()
        };

        let result = args.load_impressum_content();
        assert!(
            result.is_err(),
            "Expected error for nonexistent file, got: {:?}",
            result
        );
    }

    #[test]
    fn test_load_impressum_content_valid_file() {
        use std::io::Write;

        // Create a temporary file for testing
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_content = "Test impressum content\nLine 2 of impressum";
        write!(temp_file, "{test_content}").unwrap();
        temp_file.flush().unwrap();

        let args = Args {
            impressum_file: Some(temp_file.path().to_str().unwrap().to_string()),
            ..create_test_args()
        };

        let result = args.load_impressum_content().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_load_privacy_content_none() {
        let args = Args {
            privacy_file: None,
            ..create_test_args()
        };

        let result = args.load_privacy_content().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_privacy_content_nonexistent_file() {
        let args = Args {
            privacy_file: Some("/nonexistent/path/to/privacy.txt".to_string()),
            ..create_test_args()
        };

        let result = args.load_privacy_content();
        assert!(
            result.is_err(),
            "Expected error for nonexistent file, got: {:?}",
            result
        );
    }

    #[test]
    fn test_load_privacy_content_valid_file() {
        use std::io::Write;

        // Create a temporary file for testing
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_content = "Test privacy policy content\nData protection guidelines";
        write!(temp_file, "{test_content}").unwrap();
        temp_file.flush().unwrap();

        let args = Args {
            privacy_file: Some(temp_file.path().to_str().unwrap().to_string()),
            ..create_test_args()
        };

        let result = args.load_privacy_content().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), test_content);
    }
}
