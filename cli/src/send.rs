use std::io::{self, Read};

use anyhow::{Result, anyhow};
use colored::Colorize;
use hakanai_lib::utils::content_analysis;
use url::Url;
use zeroize::Zeroizing;

use hakanai_lib::client::Client;
use hakanai_lib::models::Payload;
use hakanai_lib::options::SecretSendOptions;

use crate::cli::SendArgs;
use crate::factory::Factory;
use crate::helper::get_user_agent_name;

pub async fn send<T: Factory>(factory: T, args: SendArgs) -> Result<()> {
    if args.ttl.as_secs() == 0 {
        return Err(anyhow!("TTL must be greater than zero seconds."));
    }

    let token = args.token()?.unwrap_or_default();
    if token.is_empty() {
        eprintln!("{}", "Warning: No token provided.".yellow());
    }

    let bytes = Zeroizing::new(read_secret(args.file.clone())?);

    if bytes.is_empty() {
        return Err(anyhow!(
            "No secret provided. Please input a secret to send."
        ));
    }
    let mut as_file = args.as_file;
    if args.file.is_some() && !as_file && content_analysis::is_binary(&bytes) {
        println!(
            "{}",
            "Sending binary files as text may lead to data corruption. Sending as file instead."
                .yellow()
        );
        as_file = true;
    }

    let filename = get_filename(args.file, as_file, args.filename)?;
    let payload = Payload::from_bytes(&bytes, filename);

    let user_agent = get_user_agent_name();
    let observer = factory.new_observer("Sending secret...")?;
    let opts = SecretSendOptions::default()
        .with_user_agent(user_agent)
        .with_observer(observer);

    let link = factory
        .new_client()
        .send_secret(args.server.clone(), payload, args.ttl, token, Some(opts))
        .await?;

    print_link(link, args.separate_key);

    Ok(())
}

fn print_link(link: Url, separate_key: bool) {
    println!("Secret sent successfully!\n");

    if separate_key {
        print_link_separate_key(link);
    } else {
        println!("Secret link: {}", link.to_string().cyan());
    }
}

fn print_link_separate_key(link: Url) {
    let mut url = link.clone();
    url.set_fragment(None);
    println!("Secret link: {}", url.to_string().cyan());

    let key = link.fragment().unwrap_or_default();
    println!("Key:         {}", key.cyan());
}

fn get_filename(
    file: Option<String>,
    as_file: bool,
    filename: Option<String>,
) -> Result<Option<String>> {
    if !as_file {
        return Ok(None);
    }

    if let Some(name) = filename {
        return Ok(Some(name));
    }

    if let Some(file_path) = file {
        Ok(std::path::Path::new(&file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string()))
    } else {
        Err(anyhow!("File name is required when sending as a file."))
    }
}

fn read_secret(file: Option<String>) -> Result<Vec<u8>> {
    if let Some(file_path) = file {
        let bytes = std::fs::read(&file_path)?;
        Ok(bytes)
    } else {
        let mut bytes: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory_mock::test_utils::MockFactory;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    use hakanai_lib::client_mock::MockClient;

    #[test]
    fn test_get_filename_not_as_file() -> Result<()> {
        let result = get_filename(Some("test.txt".to_string()), false, None)?;
        assert_eq!(result, None);
        Ok(())
    }

    #[test]
    fn test_get_filename_with_explicit_filename() -> Result<()> {
        let result = get_filename(
            Some("path/to/test.txt".to_string()),
            true,
            Some("custom.txt".to_string()),
        )?;
        assert_eq!(result, Some("custom.txt".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_from_file_path() -> Result<()> {
        let result = get_filename(Some("/path/to/test.txt".to_string()), true, None)?;
        assert_eq!(result, Some("test.txt".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_from_file_path_no_extension() -> Result<()> {
        let result = get_filename(Some("/path/to/testfile".to_string()), true, None)?;
        assert_eq!(result, Some("testfile".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_no_file_path_as_file() {
        let result = get_filename(None, true, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("File name is required")
        );
    }

    #[test]
    fn test_read_secret_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_secret.txt");
        let test_content = b"test secret content";
        fs::write(&file_path, test_content)?;

        let result = read_secret(Some(file_path.to_string_lossy().to_string()))?;
        assert_eq!(result, test_content);
        Ok(())
    }

    #[test]
    fn test_read_secret_file_not_found() {
        let result = read_secret(Some("/nonexistent/file.txt".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_secret_empty_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, b"")?;

        let result = read_secret(Some(file_path.to_string_lossy().to_string()))?;
        assert_eq!(result, b"");
        Ok(())
    }

    #[test]
    fn test_read_secret_binary_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("binary.bin");
        let binary_content = vec![0x00, 0x01, 0xFF, 0xFE, 0x42];
        fs::write(&file_path, &binary_content)?;

        let result = read_secret(Some(file_path.to_string_lossy().to_string()))?;
        assert_eq!(result, binary_content);
        Ok(())
    }

    #[tokio::test]
    async fn test_send_zero_ttl() -> Result<()> {
        let factory = MockFactory::new();
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(0))
            .with_token("token")
            .with_file(file_path.to_string_lossy().as_ref());
        let result = send(factory, args).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("TTL must be greater than zero")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_send_empty_secret_from_file() -> Result<()> {
        let factory = MockFactory::new();
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, b"")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(3600))
            .with_token("token")
            .with_file(file_path.to_string_lossy().as_ref());
        let result = send(factory, args).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No secret provided")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_send_as_file_no_file_path() -> Result<()> {
        let factory = MockFactory::new();
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, b"")?; // Empty file to test the empty secret validation

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(3600))
            .with_token("token")
            .with_file(file_path.to_string_lossy().as_ref())
            .with_as_file();
        let result = send(factory, args).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Should fail on empty secret validation
        assert!(error_msg.contains("No secret provided"));
        Ok(())
    }

    #[tokio::test]
    async fn test_send_successful_text_file() -> Result<()> {
        let expected_url = url::Url::parse("https://example.com/s/success123#key").unwrap();
        let client = MockClient::new().with_send_success(expected_url.clone());
        let factory = MockFactory::new().with_client(client);

        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test secret content")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(3600))
            .with_token("token123")
            .with_file(file_path.to_string_lossy().as_ref());
        send(factory, args).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_send_successful_as_file() -> Result<()> {
        let expected_url = url::Url::parse("https://example.com/s/file123#key").unwrap();
        let client = MockClient::new().with_send_success(expected_url.clone());
        let factory = MockFactory::new().with_client(client);

        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("document.pdf");
        fs::write(&file_path, b"fake pdf content")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(7200))
            .with_token("token456")
            .with_file(file_path.to_string_lossy().as_ref())
            .with_as_file();
        send(factory, args).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_send_successful_with_custom_filename() -> Result<()> {
        let expected_url = url::Url::parse("https://example.com/s/custom123#key").unwrap();
        let client = MockClient::new().with_send_success(expected_url.clone());
        let factory = MockFactory::new().with_client(client);

        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("original.txt");
        fs::write(&file_path, b"file content")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(3600))
            .with_token("token789")
            .with_file(file_path.to_string_lossy().as_ref())
            .with_as_file()
            .with_filename("custom_name.txt");
        send(factory, args).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_send_client_error() -> Result<()> {
        let client = MockClient::new().with_send_failure("Network error".to_string());
        let factory = MockFactory::new().with_client(client);

        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content")?;

        let args = SendArgs::builder()
            .with_server("https://example.com")
            .with_ttl(Duration::from_secs(3600))
            .with_token("token")
            .with_file(file_path.to_string_lossy().as_ref());
        let result = send(factory, args).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Network error"));
        Ok(())
    }
}
