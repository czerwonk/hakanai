// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use url::Url;

use hakanai_lib::models::CountryCode;
use hakanai_lib::utils::ip_parser::parse_ipnet;
use std::str::FromStr;
use hakanai_lib::utils::size_parser::parse_size_limit;

/// Represents the command-line arguments for the application.
#[derive(Parser)]
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

/// Represents the arguments for the `send` command.
#[derive(Debug, Clone, Parser)]
pub struct SendArgs {
    #[arg(
        short,
        long,
        default_value = "http://localhost:8080",
        env = "HAKANAI_SERVER",
        help = "Hakanai Server URL to send the secret to (eg. https://hakanai.link)."
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
        short = 'f',
        long = "file",
        help = "File to read the secret from. If not specified, reads from stdin. This can be specified multiple times to send multiple files.",
        value_name = "FILE"
    )]
    pub files: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Send the secret as a file. If not specified the type is auto determined based on the content."
    )]
    pub as_file: bool,

    #[arg(
        long,
        help = "Filename to use for the secret when sending as a file. Can be determined automatically from -f if provided for a single file."
    )]
    pub filename: Option<String>,

    #[arg(
        long,
        help = "Does not include the key in the URL fragment, but instead prints it to stdout. This is useful for sharing the key separately."
    )]
    pub separate_key: bool,

    #[arg(
        short = 'q',
        long = "qr-code",
        env = "HAKANAI_QR_CODE",
        help = "Print URL also as QR code"
    )]
    pub print_qr_code: bool,

    #[arg(
        long = "allow-ip",
        env = "HAKANAI_ALLOWED_IPS",
        help = "Comma-separated list of IP addresses (CIDR notation) that are allowed to access the secret.",
        value_parser = parse_ipnet
    )]
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,

    #[arg(
        long = "allow-country",
        env = "HAKANAI_ALLOWED_COUNTRIES",
        help = "Comma-separated list of country codes (ISO 3166-1 alpha-2) that are allowed to access the secret.",
        value_parser = CountryCode::from_str
    )]
    pub allowed_countries: Option<Vec<CountryCode>>,
}

impl SendArgs {
    /// Get the processed token, reading from file if needed
    pub fn token(&self) -> Result<Option<String>> {
        if let Some(path) = self.token_file.clone() {
            let token = self.read_token_from_file(path)?;
            Ok(Some(token))
        } else if let Some(token) = self.token.clone() {
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

    #[cfg(test)]
    pub fn builder() -> Self {
        Self {
            server: Url::parse("http://localhost:8080").unwrap(),
            ttl: Duration::from_secs(24 * 60 * 60), // 24h
            token: None,
            token_file: None,
            files: None,
            as_file: false,
            filename: None,
            separate_key: false,
            print_qr_code: false,
            allowed_ips: None,
            allowed_countries: None,
        }
    }

    #[cfg(test)]
    pub fn with_server(mut self, server: &str) -> Self {
        self.server = Url::parse(server).unwrap();
        self
    }

    #[cfg(test)]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    #[cfg(test)]
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_file(mut self, file: &str) -> Self {
        self.files = Some(vec![file.to_string()]);
        self
    }

    #[cfg(test)]
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = Some(files);
        self
    }

    #[cfg(test)]
    pub fn with_as_file(mut self) -> Self {
        self.as_file = true;
        self
    }

    #[cfg(test)]
    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_allowed_ips(mut self, allowed_ips: Vec<ipnet::IpNet>) -> Self {
        self.allowed_ips = Some(allowed_ips);
        self
    }

    #[cfg(test)]
    pub fn with_allowed_countries(mut self, allowed_countries: Vec<CountryCode>) -> Self {
        self.allowed_countries = Some(allowed_countries);
        self
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

    #[arg(
        short,
        long = "extract",
        help = "When the secret is a archive, extract its contents to the current directory."
    )]
    pub extract: bool,

    #[arg(
        short,
        long = "output-dir",
        env = "HAKANAI_OUTPUT_DIR",
        help = "Save files to this directory instead of the current one."
    )]
    pub output_dir: Option<PathBuf>,
}

impl GetArgs {
    pub fn validate(&self) -> Result<()> {
        if self.extract && self.filename.is_some() {
            return Err(anyhow!(
                "The --extract option cannot be used with --filename."
            ));
        }

        if self.to_stdout && self.filename.is_some() {
            return Err(anyhow!(
                "The --to-stdout option cannot be used with --filename."
            ));
        }

        if self.to_stdout && self.extract {
            return Err(anyhow!(
                "The --to-stdout option cannot be used with --extract."
            ));
        }

        if self.to_stdout && self.output_dir.is_some() {
            return Err(anyhow!(
                "The --to-stdout option cannot be used with --output-dir."
            ));
        }

        if let Some(ref output_dir) = self.output_dir {
            Self::validate_output_directory(output_dir)?;
        }

        Ok(())
    }

    fn validate_output_directory(output_dir: &Path) -> Result<()> {
        if !output_dir.exists() {
            return Err(anyhow!(
                "Output directory '{}' does not exist",
                output_dir.display()
            ));
        }

        if !output_dir.is_dir() {
            return Err(anyhow!(
                "Output path '{}' is not a directory",
                output_dir.display()
            ));
        }

        Ok(())
    }
}

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
        value_parser = parse_size_limit
    )]
    pub limit: Option<i64>,
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

    #[cfg(test)]
    pub fn builder(link: &str) -> Self {
        Self {
            link: Url::parse(link).unwrap(),
            key: None,
            to_stdout: false,
            filename: None,
            extract: false,
            output_dir: None,
        }
    }

    #[cfg(test)]
    pub fn with_to_stdout(mut self) -> Self {
        self.to_stdout = true;
        self
    }

    #[cfg(test)]
    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_extract(mut self) -> Self {
        self.extract = true;
        self
    }

    #[cfg(test)]
    pub fn with_output_dir(mut self, output_dir: &str) -> Self {
        self.output_dir = Some(PathBuf::from(output_dir));
        self
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

    /// Create a new user token (requires admin privileges).
    Token(TokenArgs),
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
        let get_args = GetArgs::builder("https://example.com/secret/abc123").with_to_stdout();

        assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
        assert!(get_args.to_stdout);
        assert_eq!(get_args.filename, None);
    }

    #[test]
    fn test_get_command_with_filename() {
        let get_args = GetArgs::builder("https://example.com/secret/abc123")
            .with_filename("downloaded_secret.txt");

        assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
        assert!(!get_args.to_stdout);
        assert_eq!(get_args.filename, Some("downloaded_secret.txt".to_string()));
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
        let get_args = GetArgs::builder("https://example.com/secret/abc123")
            .with_to_stdout()
            .with_filename("ignored.txt");

        assert_eq!(get_args.link.as_str(), "https://example.com/secret/abc123");
        assert!(get_args.to_stdout);
        assert_eq!(get_args.filename, Some("ignored.txt".to_string()));
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
        let send_args = SendArgs::builder().with_server("https://hakanai.link");

        assert_eq!(send_args.server.as_str(), "https://hakanai.link/");
    }

    #[test]
    fn test_send_command_with_custom_ttl() {
        let send_args = SendArgs::builder().with_ttl(Duration::from_secs(12 * 60 * 60));

        assert_eq!(send_args.ttl, Duration::from_secs(12 * 60 * 60)); // 12 hours
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
                assert_eq!(
                    send_args.files,
                    Some(vec!("/path/to/secret.txt".to_string()))
                );
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_short_file_flag() {
        let args = Args::try_parse_from(["hakanai", "send", "-f", "/tmp/data.bin"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.files, Some(vec!("/tmp/data.bin".to_string())));
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
                assert_eq!(
                    send_args.files,
                    Some(vec!("/path/to/document.pdf".to_string()))
                );
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
        let send_args = SendArgs::builder()
            .with_file("/home/user/secret.bin")
            .with_as_file()
            .with_filename("renamed_secret.bin");

        assert_eq!(
            send_args.files,
            Some(vec!("/home/user/secret.bin".to_string()))
        );
        assert!(send_args.as_file);
        assert_eq!(send_args.filename, Some("renamed_secret.bin".to_string()));
    }

    #[test]
    fn test_token_command_parsing() {
        let args = Args::try_parse_from([
            "hakanai",
            "token",
            "--server",
            "https://example.com",
            "--ttl",
            "7d",
            "--limit",
            "1048576",
        ])
        .unwrap();

        match args.command {
            Command::Token(token_args) => {
                assert_eq!(token_args.server.as_str(), "https://example.com/");
                assert_eq!(token_args.ttl, Duration::from_secs(7 * 24 * 60 * 60));
                assert_eq!(token_args.limit, Some(1048576));
            }
            _ => panic!("Expected Token command"),
        }
    }

    #[test]
    fn test_token_command_with_short_flags() {
        let args = Args::try_parse_from([
            "hakanai",
            "token",
            "-s",
            "https://hakanai.link",
            "-l",
            "5242880",
        ])
        .unwrap();

        match args.command {
            Command::Token(token_args) => {
                assert_eq!(token_args.server.as_str(), "https://hakanai.link/");
                assert_eq!(token_args.limit, Some(5242880));
            }
            _ => panic!("Expected Token command"),
        }
    }

    #[test]
    fn test_get_args_validate_success_with_defaults() {
        let args = GetArgs::builder("https://example.com/s/test#key");
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_get_args_validate_success_with_to_stdout() {
        let args = GetArgs::builder("https://example.com/s/test#key").with_to_stdout();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_get_args_validate_success_with_filename() {
        let args = GetArgs::builder("https://example.com/s/test#key").with_filename("output.txt");
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_get_args_validate_success_with_extract() {
        let args = GetArgs::builder("https://example.com/s/test#key").with_extract();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_get_args_validate_error_extract_with_filename() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_extract()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--extract option cannot be used with --filename")
        );
    }

    #[test]
    fn test_get_args_validate_error_to_stdout_with_filename() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --filename")
        );
    }

    #[test]
    fn test_get_args_validate_error_to_stdout_with_extract() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_extract();

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --extract")
        );
    }

    #[test]
    fn test_get_args_validate_error_to_stdout_with_output_dir() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_output_dir("test");

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --output-dir")
        );
    }

    #[test]
    fn test_get_args_validate_error_nonexistent_output_dir() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir("/nonexistent/directory/path");

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Output directory '/nonexistent/directory/path' does not exist")
        );
    }

    #[test]
    fn test_get_args_validate_error_output_dir_is_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path();

        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir(file_path.to_string_lossy().as_ref());

        let result = args.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("is not a directory")
        );
    }

    #[test]
    fn test_get_args_validate_success_with_valid_output_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir(temp_dir.path().to_string_lossy().as_ref());

        let result = args.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_args_validate_error_all_three_conflicting() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_extract()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(result.is_err());
        // Should fail on the first conflict check
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--extract option cannot be used with --filename")
        );
    }

    // Tests for GetArgs::secret_url()
    #[test]
    fn test_secret_url_with_fragment_in_url() {
        let args = GetArgs::builder("https://example.com/s/test#mykey");
        let url = args.secret_url().unwrap();
        assert_eq!(url.as_str(), "https://example.com/s/test#mykey");
        assert_eq!(url.fragment(), Some("mykey"));
    }

    #[test]
    fn test_secret_url_with_key_parameter() {
        let args = GetArgs::builder("https://example.com/s/test").with_key("mykey");
        let url = args.secret_url().unwrap();
        assert_eq!(url.as_str(), "https://example.com/s/test#mykey");
        assert_eq!(url.fragment(), Some("mykey"));
    }

    #[test]
    fn test_secret_url_error_no_key() {
        let args = GetArgs::builder("https://example.com/s/test");
        let result = args.secret_url();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No key provided in URL or as an argument")
        );
    }

    #[test]
    fn test_secret_url_error_both_fragment_and_key() {
        let args = GetArgs::builder("https://example.com/s/test#fragmentkey").with_key("paramkey");
        let result = args.secret_url();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("URL already contains a fragment, but a key was provided as an argument")
        );
    }

    #[test]
    fn test_secret_url_with_empty_key_parameter() {
        let args = GetArgs::builder("https://example.com/s/test").with_key("");
        let result = args.secret_url();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No key provided in URL or as an argument")
        );
    }

    #[test]
    fn test_secret_url_with_complex_url() {
        let args = GetArgs::builder("https://example.com:8080/api/v1/secret?param=value")
            .with_key("test123");
        let url = args.secret_url().unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com:8080/api/v1/secret?param=value#test123"
        );
    }

    #[test]
    fn test_secret_url_preserves_original_fragment() {
        let args = GetArgs::builder("https://example.com/s/test#original:hash");
        let url = args.secret_url().unwrap();
        assert_eq!(url.fragment(), Some("original:hash"));
    }

    #[test]
    fn test_secret_url_with_special_characters_in_key() {
        let args =
            GetArgs::builder("https://example.com/s/test").with_key("key-with_special.chars");
        let url = args.secret_url().unwrap();
        assert_eq!(url.fragment(), Some("key-with_special.chars"));
    }

    // Tests for --allow-ip flag
    #[test]
    fn test_send_command_with_single_allowed_ip() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--allow-ip", "192.168.1.100"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.100/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_ipv6_allowed_ip() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--allow-ip", "2001:db8::1"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "2001:db8::1/128");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_cidr_allowed_ip() {
        let args = Args::try_parse_from(["hakanai", "send", "--allow-ip", "10.0.0.0/8"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "10.0.0.0/8");
            }
            _ => panic!("Expected Send command"),
        }
    }

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
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 3);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
                assert_eq!(allowed_ips[1].to_string(), "10.0.0.0/8");
                assert_eq!(allowed_ips[2].to_string(), "172.16.0.100/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_ipv6_cidr_allowed_ip() {
        let args =
            Args::try_parse_from(["hakanai", "send", "--allow-ip", "2001:db8::/32"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "2001:db8::/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_mixed_ipv4_ipv6_allowed_ips() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-ip",
            "192.168.1.0/24",
            "--allow-ip",
            "2001:db8::1/128",
            "--allow-ip",
            "10.0.0.1",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 3);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
                assert_eq!(allowed_ips[1].to_string(), "2001:db8::1/128");
                assert_eq!(allowed_ips[2].to_string(), "10.0.0.1/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_without_allowed_ips() {
        let args = Args::try_parse_from(["hakanai", "send"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(send_args.allowed_ips.is_none());
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_invalid_ip_address() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-ip", "not-an-ip"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_send_command_invalid_cidr_notation() {
        let result = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-ip",
            "192.168.1.0/33", // Invalid CIDR - /33 is not valid for IPv4
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_send_command_with_allowed_ip_and_other_flags() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--server",
            "https://example.com",
            "--ttl",
            "1h",
            "--allow-ip",
            "192.168.1.0/24",
            "--allow-ip",
            "10.0.0.1",
            "--file",
            "test.txt",
            "--as-file",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.server.as_str(), "https://example.com/");
                assert_eq!(send_args.ttl, Duration::from_secs(60 * 60)); // 1 hour
                assert_eq!(send_args.files, Some(vec!["test.txt".to_string()]));
                assert!(send_args.as_file);

                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 2);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
                assert_eq!(allowed_ips[1].to_string(), "10.0.0.1/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_args_builder_with_allowed_ips() {
        use std::str::FromStr;

        let ip1 = ipnet::IpNet::from_str("192.168.1.0/24").unwrap();
        let ip2 = ipnet::IpNet::from_str("10.0.0.1/32").unwrap();

        let send_args = SendArgs::builder()
            .with_server("https://test.com")
            .with_allowed_ips(vec![ip1, ip2]);

        assert_eq!(send_args.server.as_str(), "https://test.com/");

        let allowed_ips = send_args.allowed_ips.unwrap();
        assert_eq!(allowed_ips.len(), 2);
        assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
        assert_eq!(allowed_ips[1].to_string(), "10.0.0.1/32");
    }

    #[test]
    fn test_send_command_with_localhost_allowed_ip() {
        let args = Args::try_parse_from(["hakanai", "send", "--allow-ip", "127.0.0.1"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "127.0.0.1/32");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_ipv6_localhost_allowed_ip() {
        let args = Args::try_parse_from(["hakanai", "send", "--allow-ip", "::1"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 1);
                assert_eq!(allowed_ips[0].to_string(), "::1/128");
            }
            _ => panic!("Expected Send command"),
        }
    }

    // Tests for --allow-country flag
    #[test]
    fn test_send_command_with_single_allowed_country() {
        let args = Args::try_parse_from(["hakanai", "send", "--allow-country", "US"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_countries = send_args.allowed_countries.unwrap();
                assert_eq!(allowed_countries.len(), 1);
                assert_eq!(allowed_countries[0].as_str(), "US");
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
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_countries = send_args.allowed_countries.unwrap();
                assert_eq!(allowed_countries.len(), 3);
                assert_eq!(allowed_countries[0].as_str(), "US");
                assert_eq!(allowed_countries[1].as_str(), "DE");
                assert_eq!(allowed_countries[2].as_str(), "CA");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_without_allowed_countries() {
        let args = Args::try_parse_from(["hakanai", "send"]).unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert!(send_args.allowed_countries.is_none());
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_invalid_country_code() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-country", "invalid"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_send_command_lowercase_country_code() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-country", "us"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_send_command_three_letter_country_code() {
        let result = Args::try_parse_from(["hakanai", "send", "--allow-country", "USA"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_send_command_with_allowed_countries_and_other_flags() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--server",
            "https://example.com",
            "--ttl",
            "2h",
            "--allow-country",
            "US",
            "--allow-country",
            "DE",
            "--file",
            "document.txt",
            "--as-file",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                assert_eq!(send_args.server.as_str(), "https://example.com/");
                assert_eq!(send_args.ttl, Duration::from_secs(2 * 60 * 60)); // 2 hours
                assert_eq!(send_args.files, Some(vec!["document.txt".to_string()]));
                assert!(send_args.as_file);

                let allowed_countries = send_args.allowed_countries.unwrap();
                assert_eq!(allowed_countries.len(), 2);
                assert_eq!(allowed_countries[0].as_str(), "US");
                assert_eq!(allowed_countries[1].as_str(), "DE");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_args_builder_with_allowed_countries() {
        use hakanai_lib::models::CountryCode;

        let country1 = CountryCode::new("US").unwrap();
        let country2 = CountryCode::new("DE").unwrap();

        let send_args = SendArgs::builder()
            .with_server("https://test.com")
            .with_allowed_countries(vec![country1, country2]);

        assert_eq!(send_args.server.as_str(), "https://test.com/");

        let allowed_countries = send_args.allowed_countries.unwrap();
        assert_eq!(allowed_countries.len(), 2);
        assert_eq!(allowed_countries[0].as_str(), "US");
        assert_eq!(allowed_countries[1].as_str(), "DE");
    }

    #[test]
    fn test_send_command_with_both_ip_and_country_restrictions() {
        let args = Args::try_parse_from([
            "hakanai",
            "send",
            "--allow-ip",
            "192.168.1.0/24",
            "--allow-ip",
            "10.0.0.1",
            "--allow-country",
            "US",
            "--allow-country",
            "DE",
        ])
        .unwrap();

        match args.command {
            Command::Send(send_args) => {
                let allowed_ips = send_args.allowed_ips.unwrap();
                assert_eq!(allowed_ips.len(), 2);
                assert_eq!(allowed_ips[0].to_string(), "192.168.1.0/24");
                assert_eq!(allowed_ips[1].to_string(), "10.0.0.1/32");

                let allowed_countries = send_args.allowed_countries.unwrap();
                assert_eq!(allowed_countries.len(), 2);
                assert_eq!(allowed_countries[0].as_str(), "US");
                assert_eq!(allowed_countries[1].as_str(), "DE");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_send_command_with_common_country_codes() {
        let valid_codes = ["US", "DE", "CA", "GB", "FR", "JP", "AU"];

        for code in valid_codes {
            let args = Args::try_parse_from(["hakanai", "send", "--allow-country", code]).unwrap();

            match args.command {
                Command::Send(send_args) => {
                    let allowed_countries = send_args.allowed_countries.unwrap();
                    assert_eq!(allowed_countries.len(), 1);
                    assert_eq!(allowed_countries[0].as_str(), code);
                }
                _ => panic!("Expected Send command"),
            }
        }
    }
}
