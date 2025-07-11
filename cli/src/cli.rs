use std::time::Duration;

use anyhow::{Result, anyhow};
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

/// Represents the arguments for the `send` command.
#[derive(Debug, Clone, Parser)]
pub struct SendArgs {
    #[arg(
        short,
        long,
        default_value = "http://localhost:8080",
        env = "HAKANAI_SERVER",
        help = "Hakanai Server URL to send the secret to (eg. https://hakanai.routing.rocks)."
    )]
    pub server: Url,

    #[arg(
        long,
        default_value = "24h",
        env = "HAKANAI_TTL",
        help = "Time after the secret vanishes.",
        value_parser = humantime::parse_duration,
    )]
    pub ttl: Duration,

    #[arg(
        env = "HAKANAI_TOKEN",
        help = "Token for authorization (environment variable only)."
    )]
    pub token: Option<String>,

    #[arg(
        long = "token-file",
        help = "File containing the authorization token. Environment variable HAKANAI_TOKEN takes precedence.",
        value_name = "TOKEN_FILE"
    )]
    pub token_file: Option<String>,

    #[arg(
        short,
        long,
        help = "File to read the secret from. If not specified, reads from stdin.",
        value_name = "FILE"
    )]
    pub file: Option<String>,

    #[arg(short, long, help = "Send the secret as a file.")]
    pub as_file: bool,

    #[arg(
        long,
        help = "Filename to use for the secret when sending as a file. Can be determined automatically from --file if provided."
    )]
    pub filename: Option<String>,
}

impl SendArgs {
    /// Get the processed token, reading from file if needed
    pub fn token(&self) -> Result<Option<String>> {
        if self.token.is_some() {
            // Environment variable takes precedence
            Ok(self.token.clone())
        } else if let Some(path) = self.token_file.clone() {
            let token = self.read_token_from_file(path)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn read_token_from_file(&self, path: String) -> Result<String> {
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(content.trim().to_string()),
            Err(e) => Err(anyhow!("Failed to read token file '{path}': {e}")),
        }
    }
}

/// Represents the arguments for the `get` command.
#[derive(Debug, Clone, Parser)]
pub struct GetArgs {
    pub link: Url,

    #[arg(
        short,
        long,
        help = "Optional base64 encoded secret key to use for decryption if not part of the URL."
    )]
    pub key: Option<String>,

    #[arg(
        long,
        env = "HAKANAI_TO_STDOUT",
        help = "Output the secret to stdout even if it is a file. This is useful for piping the output to other commands."
    )]
    pub to_stdout: bool,

    #[arg(
        short,
        long,
        help = "If set, the secret will be saved to a file. If the secret is a file this filename overrides the filename in the secret."
    )]
    pub filename: Option<String>,
}

impl GetArgs {
    pub fn secret_url(&self) -> Result<Url> {
        let mut url = self.link.clone();

        if url.fragment().is_some() {
            if self.key.is_some() {
                return Err(anyhow!(
                    "The URL already contains a fragment, but a key was provided as an argument."
                ));
            }

            return Ok(url);
        }

        let key = self.key.clone().unwrap_or_default();
        if key.is_empty() {
            return Err(anyhow!("No key provided in URL or as an argument"));
        }

        url.set_fragment(Some(&key));
        Ok(url)
    }
}

/// Represents the top-level command enum for the application.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Receives an ephemeral secret from the server.
    Get(GetArgs),

    /// Send a secret to the server.
    /// Content is either read from stdin or from file (if --file is specified).
    Send(SendArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::time::Duration;

    #[test]
    fn test_get_command_parsing() {
        let args =
            Args::try_parse_from(["hakanai", "get", "https://example.com/secret/abc123#test"])
                .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(
                    get_args.secret_url().unwrap().as_str(),
                    "https://example.com/secret/abc123#test"
                );
                assert!(!get_args.to_stdout);
                assert_eq!(get_args.filename, None);
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
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(
                    get_args.secret_url().unwrap().as_str(),
                    "https://example.com/secret/abc123#test"
                );
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn test_get_command_without_key() {
        let args =
            Args::try_parse_from(["hakanai", "get", "https://example.com/secret/abc123"]).unwrap();

        match args.command {
            Command::Get(get_args) => {
                let url = get_args.secret_url();
                assert!(url.is_err());
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn test_get_command_with_conflicting_keys() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123#foo",
            "-k",
            "bar",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                let url = get_args.secret_url();
                assert!(url.is_err());
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn test_get_command_with_to_stdout_flag() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "--to-stdout",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert!(get_args.to_stdout);
                assert_eq!(get_args.filename, None);
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_get_command_with_filename() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "--filename",
            "downloaded_secret.txt",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert!(!get_args.to_stdout);
                assert_eq!(get_args.filename, Some("downloaded_secret.txt".to_string()));
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_get_command_with_short_filename_flag() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "-f",
            "output.bin",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert!(!get_args.to_stdout);
                assert_eq!(get_args.filename, Some("output.bin".to_string()));
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_get_command_with_to_stdout_and_filename() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "--to-stdout",
            "--filename",
            "ignored.txt",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert!(get_args.to_stdout);
                assert_eq!(get_args.filename, Some("ignored.txt".to_string()));
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_get_command_with_all_short_flags() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "-f",
            "file.dat",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert_eq!(get_args.filename, Some("file.dat".to_string()));
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_get_command_with_special_filename() {
        let args = Args::try_parse_from([
            "hakanai",
            "get",
            "https://example.com/secret/abc123",
            "--filename",
            "path/to/file with spaces.txt",
        ])
        .unwrap();

        match args.command {
            Command::Get(get_args) => {
                assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
                assert!(!get_args.to_stdout);
                assert_eq!(
                    get_args.filename,
                    Some("path/to/file with spaces.txt".to_string())
                );
            }
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_send_command_with_custom_server() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--server",
            "https://hakanai.routing.rocks",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.server.as_str(), "https://hakanai.routing.rocks/");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_custom_ttl() {
        let args = Args::try_parse_from(["hakanai", "send", "--ttl", "12h"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.ttl, Duration::from_secs(12 * 60 * 60)); // 12 hours
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_short_flags() {
        let args =
            Args::try_parse_from(["hakanai", "send", "-s", "https://custom.server.com"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.server.as_str(), "https://custom.server.com/");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_invalid_url_parsing() {
        let result = Args::try_parse_from(["hakanai", "get", "not-a-valid-url"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ttl_parsing() {
        let result = Args::try_parse_from(["hakanai", "send", "--ttl", "invalid-duration"]);

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
            let args = Args::try_parse_from(["hakanai", "send", "--ttl", ttl_str]).unwrap();

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
        assert!(result.is_err());
    }

    #[test]
    fn test_get_command_missing_link() {
        let result = Args::try_parse_from(["hakanai", "get"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_debug_trait() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--server", "https://example.com"]).unwrap();

        let debug_output = format!("{:?}", args.command);
        assert!(debug_output.contains("Send"));
        assert!(debug_output.contains("example.com"));
    }

    #[test]
    fn test_send_command_with_token_file() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--token-file", "/path/to/token"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.token_file, Some("/path/to/token".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_all_flags() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--server",
            "https://example.com",
            "--ttl",
            "30m",
            "--token-file",
            "/path/to/token",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.server.as_str(), "https://example.com/");
                assert_eq!(send_args.ttl, Duration::from_secs(30 * 60));
                assert_eq!(send_args.token_file, Some("/path/to/token".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_file() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--file", "/path/to/secret.txt"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.file, Some("/path/to/secret.txt".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_short_file_flag() {
        let args = Args::try_parse_from(["hakanai", "send", "-f", "/tmp/data.bin"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.file, Some("/tmp/data.bin".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_as_file_flag() {
        let args = Args::try_parse_from(["hakanai", "send", "--as-file"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(send_args.as_file);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_short_as_file_flag() {
        let args = Args::try_parse_from(["hakanai", "send", "-a"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(send_args.as_file);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_without_as_file_flag() {
        let args = Args::try_parse_from(["hakanai", "send"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(!send_args.as_file);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_file_name() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--filename", "custom_name.pdf"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.filename, Some("custom_name.pdf".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_file_and_as_file() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--file",
            "/path/to/document.pdf",
            "--as-file",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.file, Some("/path/to/document.pdf".to_string()));
                assert!(send_args.as_file);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_as_file_and_custom_filename() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--as-file",
            "--filename",
            "secret_document.txt",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(send_args.as_file);
                assert_eq!(send_args.filename, Some("secret_document.txt".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_all_file_options() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--file",
            "/home/user/secret.bin",
            "--as-file",
            "--filename",
            "renamed_secret.bin",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.file, Some("/home/user/secret.bin".to_string()));
                assert!(send_args.as_file);
                assert_eq!(send_args.filename, Some("renamed_secret.bin".to_string()));
            }
            _ => panic!("Expected Send command"),
        }
    }
}
