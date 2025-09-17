// SPDX-License-Identifier: Apache-2.0

use clap::{Parser, Subcommand};

pub use crate::args::{GetArgs, SendArgs, TokenArgs};

/// Represents the command-line arguments for the application.
#[derive(Debug, Parser)]
#[command(
    version,
    name = "hakanai",
    author = "Daniel Brendgen-Czerwonk",
    about = "A minimalist one-time secret sharing web service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist.",
    after_help = "LICENSE:\n  Licensed under the Apache License, Version 2.0\n  <https://www.apache.org/licenses/LICENSE-2.0>\n\nSOURCE:\n  <https://github.com/czerwonk/hakanai>"
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

/// Represents the top-level command enum for the application.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Receives an ephemeral secret from the server.
    Get(GetArgs),

    /// Send a secret to the server.
    /// Content is either read from stdin or from file (if --file is specified).
    Send(SendArgs),

    /// Create a new user token (requires admin privileges).
    Token(TokenArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use clap::Parser;

    #[test]
    fn test_get_command_parsing() {
        let args =
            Args::try_parse_from(["hakanai", "get", "https://example.com/secret/abc123#test"])
                .expect("Failed to parse arguments");

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(
                    get_args
                        .secret_url()
                        .expect("Failed to get secret URL")
                        .as_str(),
                    "https://example.com/secret/abc123#test"
                );
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn test_get_command_with_key_arg() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "--key",
            "test",
        ])
        .expect("Failed to parse arguments");

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(
                    get_args
                        .secret_url()
                        .expect("Failed to get secret URL")
                        .as_str(),
                    "https://example.com/secret/abc123#test"
                );
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn test_invalid_url_parsing() {
        let result = Args::try_parse_from(["hakanai", "get", "not-a-valid-url"]);

        assert!(
            result.is_err(),
            "Expected error for invalid URL, got: {:?}",
            result
        );
    }

    #[test]
    fn test_invalid_ttl_parsing() {
        let result = Args::try_parse_from(["hakanai", "send", "--ttl", "invalid-duration"]);

        assert!(
            result.is_err(),
            "Expected error for invalid TTL, got: {:?}",
            result
        );
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
            let args = Args::try_parse_from(["hakanai", "send", "--ttl", ttl_str])
                .expect("Failed to parse arguments");

            match args.command {
                Command::Send(send_args) => {
                    assert_eq!(
                        send_args.ttl, expected_duration,
                        "Failed for TTL: {ttl_str}"
                    );
                }
                _ => panic!("Expected Send command for TTL: {ttl_str}"),
            }
        }
    }

    #[test]
    fn test_missing_subcommand() {
        let result = Args::try_parse_from(["hakanai"]);
        assert!(
            result.is_err(),
            "Expected error for missing subcommand, got: {:?}",
            result
        );
    }

    #[test]
    fn test_get_command_missing_link() {
        let result = Args::try_parse_from(["hakanai", "get"]);
        assert!(
            result.is_err(),
            "Expected error for missing link, got: {:?}",
            result
        );
    }

    // Tests for multi-assignment with separators (prevent regressions)
    #[test]
    fn test_send_command_with_multiple_allowed_ips() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-ip",
            "192.168.1.0/24",
            "--allow-ip",
            "10.0.0.0/8",
            "--allow-ip",
            "172.16.0.100",
        ])
        .expect("Failed to parse arguments");

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.expect("Allowed IPs should be set");
                assert_eq!(allowed_ips.len(), 3);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
                assert_eq!(allowed_ips[1].to_string(), "10.0.0.0/8");
                assert_eq!(allowed_ips[2].to_string(), "172.16.0.100/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_multiple_allowed_countries() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-country",
            "US",
            "--allow-country",
            "DE",
            "--allow-country",
            "CA",
        ])
        .expect("Failed to parse arguments");

        match args.command {
            Command::Send(send_args) => {
                let allowed_countries = send_args
                    .allowed_countries
                    .expect("Allowed countries should be set");
                assert_eq!(allowed_countries.len(), 3);
                assert_eq!(allowed_countries[0].as_str(), "US");
                assert_eq!(allowed_countries[1].as_str(), "DE");
                assert_eq!(allowed_countries[2].as_str(), "CA");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_comma_separated_asns() {
        let args = Args::try_parse_from(["hakanai", "send", "--allow-asn", "13335,15169,32934"])
            .expect("Failed to parse arguments");

        match args.command {
            Command::Send(send_args) => {
                let allowed_asns = send_args.allowed_asns.expect("Allowed ASNs should be set");
                assert_eq!(allowed_asns.len(), 3);
                assert_eq!(allowed_asns[0], 13335);
                assert_eq!(allowed_asns[1], 15169);
                assert_eq!(allowed_asns[2], 32934);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_invalid_ip_address() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-ip", "not-an-ip"]);
        assert!(result.is_err(), "Expected error, got: {:?}", result);
    }

    #[test]
    fn test_send_command_invalid_cidr_notation() {
        let result = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-ip",
            "192.168.1.0/33", // Invalid CIDR - /33 is not valid for IPv4
        ]);
        assert!(result.is_err(), "Expected error, got: {:?}", result);
    }

    #[test]
    fn test_send_command_invalid_country_code() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-country", "invalid"]);
        assert!(result.is_err(), "Expected error, got: {:?}", result);
    }

    #[test]
    fn test_send_command_invalid_asn_non_numeric() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-asn", "invalid"]);
        assert!(result.is_err(), "Expected error, got: {:?}", result);
    }

    #[test]
    fn test_send_command_invalid_asn_too_large() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-asn", "4294967296"]); // u32::MAX + 1
        assert!(result.is_err(), "Expected error, got: {:?}", result);
    }
}
