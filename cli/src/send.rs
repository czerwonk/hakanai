use core::clone::Clone;
use core::convert::AsRef;
use std::io::{self, Cursor, Read, Write};

use anyhow::{Result, anyhow};
use colored::Colorize;
use hakanai_lib::utils::content_analysis;
use qrcode::{QrCode, render::unicode};
use url::Url;
use zeroize::{Zeroize, Zeroizing};
use zip::{ZipWriter, write::ExtendedFileOptions, write::FileOptions};

use hakanai_lib::client::Client;
use hakanai_lib::models::Payload;
use hakanai_lib::options::SecretSendOptions;
use hakanai_lib::timestamp;

use crate::cli::SendArgs;
use crate::factory::Factory;
use crate::helper::get_user_agent_name;

struct Secret {
    bytes: Zeroizing<Vec<u8>>,
    filename: Option<String>,
}

pub async fn send<T: Factory>(factory: T, args: SendArgs) -> Result<()> {
    if args.ttl.as_secs() == 0 {
        return Err(anyhow!("TTL must be greater than zero seconds."));
    }

    let token = args.token()?.unwrap_or_default();
    if token.is_empty() {
        eprintln!("{}", "Warning: No token provided.".yellow());
    }

    let secret = read_secret(args.clone())?;
    if secret.bytes.is_empty() {
        return Err(anyhow!(
            "No secret provided. Please input a secret to send."
        ));
    }

    let filename = get_filename(&secret, args.clone())?;
    let payload = Payload::from_bytes(secret.bytes.as_ref(), filename);

    let user_agent = get_user_agent_name();
    let observer = factory.new_observer("Sending secret...")?;
    let opts = SecretSendOptions::default()
        .with_user_agent(user_agent)
        .with_observer(observer);

    let mut link = factory
        .new_client()
        .send_secret(args.server.clone(), payload, args.ttl, token, Some(opts))
        .await?
        .clone();

    print_link(&mut link, args)?;

    Ok(())
}

fn read_secret(args: SendArgs) -> Result<Secret> {
    if let Some(files) = args.files {
        read_secret_from_files(files)
    } else {
        let mut bytes = Zeroizing::new(Vec::new());
        io::stdin().read_to_end(&mut bytes)?;
        Ok(Secret {
            bytes,
            filename: None,
        })
    }
}

fn read_secret_from_files(files: Vec<String>) -> Result<Secret> {
    if files.len() != 1 {
        return archive_files(files);
    }

    let file_path = files[0].clone();
    let bytes = Zeroizing::new(std::fs::read(&file_path)?);
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string());

    Ok(Secret { bytes, filename })
}

fn archive_files(files: Vec<String>) -> Result<Secret> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);

    let mut zip = ZipWriter::new(cursor);
    for file in files {
        let bytes = Zeroizing::new(std::fs::read(&file)?);
        let filename = std::path::Path::new(&file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        zip.start_file(filename, FileOptions::<ExtendedFileOptions>::default())?;
        zip.write_all(bytes.as_ref())?;
    }

    zip.finish()?;

    let timestamp = timestamp::now_string()?;
    let filename = format!("{timestamp}.zip");

    Ok(Secret {
        bytes: Zeroizing::new(buffer),
        filename: Some(filename),
    })
}

fn get_filename(secret: &Secret, args: SendArgs) -> Result<Option<String>> {
    let mut as_file = args.as_file;
    if args.files.is_some() && !as_file && content_analysis::is_binary(secret.bytes.as_ref()) {
        println!(
            "{}",
            "Sending binary files as text may lead to data corruption. Sending as file instead."
                .yellow()
        );
        as_file = true;
    }

    if !as_file {
        return Ok(None);
    }

    if let Some(filename) = args.filename {
        return Ok(Some(filename));
    }

    if let Some(filename) = &secret.filename {
        return Ok(Some(filename.clone()));
    } else {
        Err(anyhow!("File name is required when sending as a file."))
    }
}

fn print_link(link: &mut Url, args: SendArgs) -> Result<()> {
    println!("Secret sent successfully!\n");

    if args.no_hash {
        remove_hash(link);
    }

    if args.separate_key {
        print_link_separate_key(link);
    } else {
        println!("Secret link: {}", link.to_string().cyan());
    }

    if args.print_qr_code {
        print_qr_code(link)?;
    }

    Ok(())
}

fn remove_hash(link: &mut Url) {
    let fragment = link
        .fragment()
        .unwrap_or_default()
        .split(":")
        .collect::<Vec<&str>>();

    let mut key = fragment[0].to_string();
    link.set_fragment(Some(&key));
    key.zeroize();
}

fn print_qr_code(link: &Url) -> Result<()> {
    let code = QrCode::with_error_correction_level(link.to_string(), qrcode::EcLevel::L)?;
    let ascii = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    println!("{ascii}");

    Ok(())
}

fn print_link_separate_key(link: &mut Url) {
    let mut fragment = link.fragment().unwrap_or_default().to_string();
    link.set_fragment(None);

    println!("Secret link: {}", link.to_string().cyan());
    println!("Key:         {}", fragment.cyan());

    fragment.zeroize();
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
        let args = SendArgs::builder().with_filename("test.txt");
        let secret = Secret {
            bytes: Zeroizing::new(b"test content".to_vec()),
            filename: None,
        };
        let result = get_filename(&secret, args)?;
        assert_eq!(result, None);
        Ok(())
    }

    #[test]
    fn test_get_filename_with_explicit_filename() -> Result<()> {
        let args = SendArgs::builder()
            .with_as_file()
            .with_filename("custom.txt");
        let secret = Secret {
            bytes: Zeroizing::new(b"test content".to_vec()),
            filename: Some("path/to/test.txt".to_string()),
        };
        let result = get_filename(&secret, args)?;
        assert_eq!(result, Some("custom.txt".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_from_secret() -> Result<()> {
        let args = SendArgs::builder().with_as_file();
        let secret = Secret {
            bytes: Zeroizing::new(b"test content".to_vec()),
            filename: Some("test.txt".to_string()),
        };
        let result = get_filename(&secret, args)?;
        assert_eq!(result, Some("test.txt".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_from_file_path_no_extension() -> Result<()> {
        let args = SendArgs::builder().with_as_file();
        let secret = Secret {
            bytes: Zeroizing::new(b"test content".to_vec()),
            filename: Some("testfile".to_string()),
        };
        let result = get_filename(&secret, args)?;
        assert_eq!(result, Some("testfile".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_filename_no_file_path_as_file() {
        let args = SendArgs::builder().with_as_file();
        let secret = Secret {
            bytes: Zeroizing::new(b"test content".to_vec()),
            filename: None,
        };
        let result = get_filename(&secret, args);
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

        let args = SendArgs::builder().with_file(file_path.to_string_lossy().as_ref());
        let result = read_secret(args)?;
        assert_eq!(result.bytes.to_vec(), test_content);
        Ok(())
    }

    #[test]
    fn test_read_secret_from_files_creates_archive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_secret.txt");
        let test_content = b"test secret content";
        fs::write(&file_path, test_content)?;

        let file_path2 = temp_dir.path().join("test_secret2.txt");
        let test_content2 = b"test secret content part 2";
        fs::write(&file_path2, test_content2)?;

        let args = SendArgs::builder().with_files(vec![
            file_path.to_string_lossy().as_ref().to_string(),
            file_path2.to_string_lossy().as_ref().to_string(),
        ]);
        let result = read_secret(args)?;
        assert_eq!(&result.bytes[0..4], b"PK\x03\x04", "Invalid ZIP signature");

        if let Some(filename) = &result.filename {
            assert!(filename.ends_with(".zip"), "Filename should end with .zip");
        } else {
            panic!("Filename should be set for archive");
        }

        Ok(())
    }

    #[test]
    fn test_read_secret_file_not_found() {
        let args = SendArgs::builder().with_file("/nonexistent/file.txt");
        let result = read_secret(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_secret_empty_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, b"")?;

        let args = SendArgs::builder().with_file(file_path.to_string_lossy().as_ref());
        let result = read_secret(args)?;
        assert_eq!(result.bytes.to_vec(), b"");
        Ok(())
    }

    #[test]
    fn test_read_secret_binary_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("binary.bin");
        let binary_content = vec![0x00, 0x01, 0xFF, 0xFE, 0x42];
        fs::write(&file_path, &binary_content)?;

        let args = SendArgs::builder().with_file(file_path.to_string_lossy().as_ref());
        let result = read_secret(args)?;
        assert_eq!(result.bytes.to_vec(), binary_content);
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
