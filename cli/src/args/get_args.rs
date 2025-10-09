// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::Parser;
use url::Url;

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
        help = "Ask for decryption key if the URL does not contain a key in fragment."
    )]
    pub ask_key: bool,

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

    #[arg(
        short,
        long,
        help = "If the secret is protected by a passphrase, provide it here."
    )]
    pub passphrase: Option<String>,

    #[arg(long, help = "Ask for passphrase protecting the secret.")]
    pub ask_passphrase: bool,
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

        if self.passphrase.is_some() && self.ask_passphrase {
            return Err(anyhow!(
                "The --passphrase option cannot be used with --ask-passphrase."
            ));
        }

        if self.key.is_some() && self.ask_key {
            return Err(anyhow!("The --key option cannot be used with --ask-key."));
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

        let key = if self.ask_key {
            rpassword::prompt_password("Enter decryption key: ")?
        } else {
            self.key.clone().unwrap_or_default()
        };

        if key.is_empty() {
            return Err(anyhow!("No decryption key provided"));
        }

        url.set_fragment(Some(&key));
        Ok(url)
    }

    #[cfg(test)]
    pub fn builder(link: &str) -> Self {
        Self {
            link: Url::parse(link).expect("Invalid URL"),
            key: None,
            to_stdout: false,
            filename: None,
            extract: false,
            output_dir: None,
            passphrase: None,
            ask_key: false,
            ask_passphrase: false,
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

    #[cfg(test)]
    pub fn with_passphrase(mut self, passphrase: &str) -> Self {
        self.passphrase = Some(passphrase.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_ask_key(mut self) -> Self {
        self.ask_key = true;
        self
    }

    #[cfg(test)]
    pub fn with_ask_passphrase(mut self) -> Self {
        self.ask_passphrase = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_success_with_defaults() -> Result<()> {
        let args = GetArgs::builder("https://example.com/s/test#key");
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_success_with_to_stdout() -> Result<()> {
        let args = GetArgs::builder("https://example.com/s/test#key").with_to_stdout();
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_success_with_filename() -> Result<()> {
        let args = GetArgs::builder("https://example.com/s/test#key").with_filename("output.txt");
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_success_with_extract() -> Result<()> {
        let args = GetArgs::builder("https://example.com/s/test#key").with_extract();
        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_error_extract_with_filename() -> Result<()> {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_extract()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(
            result.is_err(),
            "Expected validation error, got: {:?}",
            result
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("--extract option cannot be used with --filename"),
            "Error message doesn't contain expected text: {}",
            error_msg
        );
        Ok(())
    }

    #[test]
    fn test_validate_error_to_stdout_with_filename() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --filename")
        );
    }

    #[test]
    fn test_validate_error_to_stdout_with_extract() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_extract();

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --extract")
        );
    }

    #[test]
    fn test_validate_error_to_stdout_with_output_dir() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_output_dir("test");

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--to-stdout option cannot be used with --output-dir")
        );
    }

    #[test]
    fn test_validate_error_nonexistent_output_dir() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir("/nonexistent/directory/path");

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Output directory '/nonexistent/directory/path' does not exist")
        );
    }

    #[test]
    fn test_validate_error_output_dir_is_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path();

        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir(file_path.to_string_lossy().as_ref());

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("is not a directory")
        );
    }

    #[test]
    fn test_validate_success_with_valid_output_dir() -> Result<()> {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_output_dir(temp_dir.path().to_string_lossy().as_ref());

        args.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_error_all_three_conflicting() {
        let args = GetArgs::builder("https://example.com/s/test#key")
            .with_to_stdout()
            .with_extract()
            .with_filename("output.txt");

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        // Should fail on the first conflict check
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--extract option cannot be used with --filename")
        );
    }

    #[test]
    fn test_validate_key_conflicting() {
        let args = GetArgs::builder("https://example.com/s/test")
            .with_key("key")
            .with_ask_key();

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--key option cannot be used with --ask-key.")
        );
    }

    #[test]
    fn test_validate_passphrase_conflicting() {
        let args = GetArgs::builder("https://example.com/s/test")
            .with_passphrase("passphrase")
            .with_ask_passphrase();

        let result = args.validate();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("--passphrase option cannot be used with --ask-passphrase.")
        );
    }

    #[test]
    fn test_secret_url_with_fragment_in_url() {
        let args = GetArgs::builder("https://example.com/s/test#mykey");
        let url = args.secret_url().expect("Failed to get secret URL");
        assert_eq!(url.as_str(), "https://example.com/s/test#mykey");
        assert_eq!(url.fragment(), Some("mykey"));
    }

    #[test]
    fn test_secret_url_with_key_parameter() {
        let args = GetArgs::builder("https://example.com/s/test").with_key("mykey");
        let url = args.secret_url().expect("Failed to get secret URL");
        assert_eq!(url.as_str(), "https://example.com/s/test#mykey");
        assert_eq!(url.fragment(), Some("mykey"));
    }

    #[test]
    fn test_secret_url_error_no_key() {
        let args = GetArgs::builder("https://example.com/s/test");
        let result = args.secret_url();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No decryption key provided")
        );
    }

    #[test]
    fn test_secret_url_error_both_fragment_and_key() {
        let args = GetArgs::builder("https://example.com/s/test#fragmentkey").with_key("paramkey");
        let result = args.secret_url();
        assert!(result.is_err(), "Expected error, got: {:?}", result);
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
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No decryption key provided")
        );
    }

    #[test]
    fn test_secret_url_with_complex_url() {
        let args = GetArgs::builder("https://example.com:8080/api/v1/secret?param=value")
            .with_key("test123");
        let url = args.secret_url().expect("Failed to get secret URL");
        assert_eq!(
            url.as_str(),
            "https://example.com:8080/api/v1/secret?param=value#test123"
        );
    }

    #[test]
    fn test_secret_url_preserves_original_fragment() {
        let args = GetArgs::builder("https://example.com/s/test#original:hash");
        let url = args.secret_url().expect("Failed to get secret URL");
        assert_eq!(url.fragment(), Some("original:hash"));
    }

    #[test]
    fn test_secret_url_with_special_characters_in_key() {
        let args =
            GetArgs::builder("https://example.com/s/test").with_key("key-with_special.chars");
        let url = args.secret_url().expect("Failed to get secret URL");
        assert_eq!(url.fragment(), Some("key-with_special.chars"));
    }
}
