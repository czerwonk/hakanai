use std::time::Duration;

use clap::{Parser, Subcommand};
use url::Url;

/// Represents the command-line arguments for the application.
#[derive(Parser)]
#[command(
    version,
    name = "hakanai",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist."
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Receives an ephemeral secret from the server.
    Get { link: Url },

    /// Send a secret to the server.
    /// Content is read from stdin.
    Send {
        #[arg(
            short,
            long,
            default_value = "http://localhost:8080",
            help = "Hakanai Server URL to send the secret to (eg. https://hakanai.routing.rocks)."
        )]
        server: Url,

        #[arg(
            short,
            long,
            default_value = "24h",
            help = "Time after the secret vanishes.",
            value_parser = humantime::parse_duration,
        )]
        ttl: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::time::Duration;

    #[test]
    fn test_get_command_parsing() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "get",
            "https://example.com/secret/abc123"
        ]).unwrap();

        match args.command {
            Command::Get { link } => {
                assert_eq!(link.as_str(), "https://example.com/secret/abc123");
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_send_command_with_defaults() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "send"
        ]).unwrap();

        match args.command {
            Command::Send { server, ttl } => {
                assert_eq!(server.as_str(), "http://localhost:8080/");
                assert_eq!(ttl, Duration::from_secs(24 * 60 * 60)); // 24 hours
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_custom_server() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "send",
            "--server", "https://hakanai.routing.rocks"
        ]).unwrap();

        match args.command {
            Command::Send { server, ttl: _ } => {
                assert_eq!(server.as_str(), "https://hakanai.routing.rocks/");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_custom_ttl() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "send",
            "--ttl", "12h"
        ]).unwrap();

        match args.command {
            Command::Send { server: _, ttl } => {
                assert_eq!(ttl, Duration::from_secs(12 * 60 * 60)); // 12 hours
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_short_flags() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "send",
            "-s", "https://custom.server.com",
            "-t", "30m"
        ]).unwrap();

        match args.command {
            Command::Send { server, ttl } => {
                assert_eq!(server.as_str(), "https://custom.server.com/");
                assert_eq!(ttl, Duration::from_secs(30 * 60)); // 30 minutes
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_invalid_url_parsing() {
        let result = Args::try_parse_from(&[
            "hakanai",
            "get",
            "not-a-valid-url"
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ttl_parsing() {
        let result = Args::try_parse_from(&[
            "hakanai",
            "send",
            "--ttl", "invalid-duration"
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_various_ttl_formats() {
        let test_cases = vec![
            ("1s", Duration::from_secs(1)),
            ("5m", Duration::from_secs(5 * 60)),
            ("2h", Duration::from_secs(2 * 60 * 60)),
            ("3d", Duration::from_secs(3 * 24 * 60 * 60)),
            ("1w", Duration::from_secs(7 * 24 * 60 * 60)),
        ];

        for (ttl_str, expected_duration) in test_cases {
            let args = Args::try_parse_from(&[
                "hakanai",
                "send",
                "--ttl", ttl_str
            ]).unwrap();

            match args.command {
                Command::Send { server: _, ttl } => {
                    assert_eq!(ttl, expected_duration, "Failed for TTL: {}", ttl_str);
                }
                _ => panic!("Expected Send command for TTL: {}", ttl_str),
            }
        }
    }

    #[test]
    fn test_missing_subcommand() {
        let result = Args::try_parse_from(&["hakanai"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_command_missing_link() {
        let result = Args::try_parse_from(&["hakanai", "get"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_debug_trait() {
        let args = Args::try_parse_from(&[
            "hakanai",
            "send",
            "--server", "https://example.com"
        ]).unwrap();

        let debug_output = format!("{:?}", args.command);
        assert!(debug_output.contains("Send"));
        assert!(debug_output.contains("example.com"));
    }
}