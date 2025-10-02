// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;
use std::time::Duration;

use anyhow::{Result, anyhow};
use clap::Parser;
use url::Url;

use hakanai_lib::models::{CountryCode, SecretRestrictions};
use zeroize::Zeroizing;

use hakanai_lib::utils::ip;

#[cfg(test)]
use hakanai_lib::utils::test::MustParse;

const MIN_PASSPHRASE_LENGTH: usize = 8;

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
        value_delimiter = ',',
        value_parser = ip::parse_ipnet,
    )]
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,

    #[arg(
        long = "allow-country",
        env = "HAKANAI_ALLOWED_COUNTRIES",
        help = "Comma-separated list of country codes (ISO 3166-1 alpha-2) that are allowed to access the secret.",
        value_delimiter = ',',
        value_parser = CountryCode::from_str
    )]
    pub allowed_countries: Option<Vec<CountryCode>>,

    #[arg(
        long = "allow-asn",
        env = "HAKANAI_ALLOWED_ASNS",
        help = "Comma-separated list of automomous systems that are allowed to access the secret.",
        value_delimiter = ','
    )]
    pub allowed_asns: Option<Vec<u32>>,

    #[arg(
        short = 'p',
        long,
        help = "If set, the passphrase will be required to access the secret. The passphrase is not part of the URL and must be shared separately.",
        env = "HAKANAI_REQUIRE_PASSPHRASE"
    )]
    pub require_passphrase: Option<String>,
}

impl SendArgs {
    pub fn validate(&self) -> Result<()> {
        if let Some(passphrase) = &self.require_passphrase
            && passphrase.trim().chars().count() < MIN_PASSPHRASE_LENGTH
        {
            return Err(anyhow!(format!(
                "The passphrase must be at least {MIN_PASSPHRASE_LENGTH} characters long if set."
            )));
        }

        Ok(())
    }

    pub fn get_restrictions(&self) -> Option<SecretRestrictions> {
        let mut restrictions = SecretRestrictions::default();

        if let Some(allowed_ips) = &self.allowed_ips {
            restrictions = restrictions.with_allowed_ips(allowed_ips.clone());
        }

        if let Some(allowed_countries) = &self.allowed_countries {
            restrictions = restrictions.with_allowed_countries(allowed_countries.clone());
        }

        if let Some(allowed_asns) = &self.allowed_asns {
            restrictions = restrictions.with_allowed_asns(allowed_asns.clone());
        }

        if let Some(ref passphrase) = self.require_passphrase
            && !passphrase.is_empty()
        {
            let bytes = Zeroizing::new(passphrase.bytes().collect::<Vec<u8>>());
            restrictions = restrictions.with_passphrase(&bytes);
        }

        if restrictions.is_empty() {
            None
        } else {
            Some(restrictions)
        }
    }

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
            server: "http://localhost:8080".must_parse(),
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
            allowed_asns: None,
            require_passphrase: None,
        }
    }

    #[cfg(test)]
    pub fn with_server(mut self, server: &str) -> Self {
        self.server = server.must_parse();
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

    #[cfg(test)]
    pub fn with_allowed_asns(mut self, allowed_asns: Vec<u32>) -> Self {
        self.allowed_asns = Some(allowed_asns);
        self
    }

    #[cfg(test)]
    pub fn with_require_passphrase(mut self, passphrase: &str) -> Self {
        self.require_passphrase = Some(passphrase.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hakanai_lib::utils::test::MustParse;

    #[test]
    fn test_validate_passphrase_exactly_8_chars() -> Result<()> {
        let args = SendArgs::builder().with_require_passphrase("12345678");
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_passphrase_more_than_8_chars() -> Result<()> {
        let args = SendArgs::builder().with_require_passphrase("123456789");
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_passphrase_7_chars_fails() {
        let args = SendArgs::builder().with_require_passphrase("1234567");
        let result = args.validate();

        assert!(result.is_err(), "Expected error for 7-char passphrase");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("passphrase must be at least 8 characters"),
            "Error message should mention 8 character minimum"
        );
    }

    #[test]
    fn test_validate_passphrase_empty_fails() {
        let args = SendArgs::builder().with_require_passphrase("");
        let result = args.validate();

        assert!(result.is_err(), "Expected error for empty passphrase");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("passphrase must be at least 8 characters"),
            "Error message should mention 8 character minimum"
        );
    }

    #[test]
    fn test_validate_passphrase_8_spaces_fails() {
        // 8 spaces should fail validation because trimmed length is 0
        let args = SendArgs::builder().with_require_passphrase("        "); // 8 spaces
        let result = args.validate();

        assert!(
            result.is_err(),
            "Expected error for 8 spaces (trimmed to empty)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("passphrase must be at least 8 characters"),
            "Error message should mention 8 character minimum"
        );
    }

    #[test]
    fn test_validate_passphrase_with_leading_trailing_spaces() -> Result<()> {
        // Should validate based on trimmed length
        let args = SendArgs::builder().with_require_passphrase("  123456  ");
        let result = args.validate();

        assert!(
            result.is_err(),
            "Expected error for passphrase with trimmed length < 8"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("passphrase must be at least 8 characters"),
            "Error message should mention 8 character minimum"
        );
        Ok(())
    }

    #[test]
    fn test_validate_passphrase_unicode_8_chars() -> Result<()> {
        // 8 Unicode characters (not bytes) should pass
        let args = SendArgs::builder().with_require_passphrase("パスワード四文字");
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_passphrase_unicode_7_chars_fails() {
        // 7 Unicode characters should fail
        let args = SendArgs::builder().with_require_passphrase("パスワード三文");
        let result = args.validate();

        assert!(
            result.is_err(),
            "Expected error for 7 Unicode char passphrase"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("passphrase must be at least 8 characters"),
            "Error message should mention 8 character minimum"
        );
    }

    #[test]
    fn test_validate_no_passphrase() -> Result<()> {
        // No passphrase should pass validation (it's optional)
        let args = SendArgs::builder();
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_get_restrictions_with_all_options() {
        // Test that all restriction types are properly processed when set
        let args = SendArgs::builder()
            .with_allowed_ips(vec!["192.168.1.0/24".must_parse()])
            .with_allowed_countries(vec!["US".must_parse()])
            .with_allowed_asns(vec![13335])
            .with_require_passphrase("test123456");

        let result = args.get_restrictions();
        let restrictions = result.expect("Restictions should be set");
        assert_eq!(
            restrictions
                .allowed_ips
                .as_ref()
                .expect("Allowed IPs should be set")
                .len(),
            1,
            "Should have one IP restriction"
        );
        assert_eq!(
            restrictions
                .allowed_countries
                .as_ref()
                .expect("Allowed countries should be set")
                .len(),
            1,
            "Should have one country restriction"
        );
        assert_eq!(
            restrictions
                .allowed_asns
                .as_ref()
                .expect("Allowed ASNs should be set")
                .len(),
            1,
            "Should have one ASN restriction"
        );
        assert!(
            restrictions.passphrase_hash.is_some(),
            "Should have passphrase restriction"
        );
    }

    #[test]
    fn test_get_restrictions_empty() {
        // Test that no restrictions returns None
        let args = SendArgs::builder();
        let result = args.get_restrictions();
        assert!(
            result.is_none(),
            "Should return None when no restrictions are set"
        );
    }

    #[test]
    fn test_get_restrictions_short_passphrase_ignored() {
        // Test that empty passphrase is ignored and no restrictions are created
        let args = SendArgs::builder().with_require_passphrase("");
        let result = args.get_restrictions();
        assert!(
            result.is_none(),
            "Should return None when passphrase is empty"
        );
    }

    #[test]
    fn test_get_restrictions_only_ips() {
        // Test that only IP restrictions are processed correctly
        let args = SendArgs::builder().with_allowed_ips(vec!["10.0.0.0/8".must_parse()]);

        let result = args.get_restrictions();
        let restrictions = result.expect("Should have restrictions");
        assert_eq!(
            restrictions
                .allowed_ips
                .as_ref()
                .expect("Allowed IPs should be set")
                .len(),
            1,
            "Should have one IP restriction"
        );
        assert!(
            restrictions.allowed_countries.is_none(),
            "Should have no country restrictions"
        );
        assert!(
            restrictions.allowed_asns.is_none(),
            "Should have no ASN restrictions"
        );
        assert!(
            restrictions.passphrase_hash.is_none(),
            "Should have no passphrase restriction"
        );
    }

    #[test]
    fn test_get_restrictions_only_countries() {
        // Test that only country restrictions are processed correctly
        let args = SendArgs::builder().with_allowed_countries(vec!["DE".must_parse()]);

        let result = args.get_restrictions();
        let restrictions = result.expect("Restrictions should be set");
        assert!(
            restrictions.allowed_ips.is_none(),
            "Should have no IP restrictions"
        );
        assert_eq!(
            restrictions
                .allowed_countries
                .as_ref()
                .expect("Allowed contries should be set")
                .len(),
            1,
            "Should have one country restriction"
        );
        assert!(
            restrictions.allowed_asns.is_none(),
            "Should have no ASN restrictions"
        );
        assert!(
            restrictions.passphrase_hash.is_none(),
            "Should have no passphrase restriction"
        );
    }

    #[test]
    fn test_get_restrictions_only_asns() {
        // Test that only ASN restrictions are processed correctly
        let args = SendArgs::builder().with_allowed_asns(vec![15169]);

        let result = args.get_restrictions();
        let restrictions = result.expect("Restrictions should be set");
        assert!(
            restrictions.allowed_ips.is_none(),
            "Should have no IP restrictions"
        );
        assert!(
            restrictions.allowed_countries.is_none(),
            "Should have no country restrictions"
        );
        assert_eq!(
            restrictions
                .allowed_asns
                .as_ref()
                .expect("Allowed ASNs should be set")
                .len(),
            1,
            "Should have one ASN restriction"
        );
        assert!(
            restrictions.passphrase_hash.is_none(),
            "Should have no passphrase restriction"
        );
    }

    #[test]
    fn test_get_restrictions_only_passphrase() {
        // Test that only passphrase restrictions are processed correctly
        let args = SendArgs::builder().with_require_passphrase("validpassword");

        let result = args.get_restrictions();
        let restrictions = result.expect("Restrictions should be set");
        assert!(
            restrictions.allowed_ips.is_none(),
            "Should have no IP restrictions"
        );
        assert!(
            restrictions.allowed_countries.is_none(),
            "Should have no country restrictions"
        );
        assert!(
            restrictions.allowed_asns.is_none(),
            "Should have no ASN restrictions"
        );
        assert!(
            restrictions.passphrase_hash.is_some(),
            "Should have passphrase restriction"
        );
    }
}
