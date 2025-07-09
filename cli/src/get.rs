use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use colored::Colorize;
use zeroize::Zeroizing;

use hakanai_lib::client::Client;
use hakanai_lib::options::SecretReceiveOptions;
use hakanai_lib::timestamp;

use crate::factory::Factory;
use crate::helper::get_user_agent_name;

pub async fn get<T: Factory>(
    factory: T,
    link: url::Url,
    to_stdout: bool,
    filename: Option<String>,
) -> Result<()> {
    let user_agent = get_user_agent_name();
    let observer = factory.new_observer("Receiving secret...")?;
    let opts = SecretReceiveOptions::default()
        .with_user_agent(user_agent)
        .with_observer(observer);

    let payload = factory
        .new_client()
        .receive_secret(link.clone(), Some(opts))
        .await?;

    let bytes = Zeroizing::new(payload.decode_bytes()?);
    let filename = filename.or_else(|| payload.filename.clone());
    output_secret(&bytes, to_stdout, filename)?;

    Ok(())
}

fn output_secret(bytes: &[u8], to_stdout: bool, filename: Option<String>) -> Result<()> {
    if to_stdout {
        print_to_stdout(bytes)?;
    } else if let Some(file) = filename {
        write_to_file(file, bytes)?;
    } else {
        print_to_stdout(bytes)?;
    }

    Ok(())
}

fn print_to_stdout(bytes: &[u8]) -> Result<()> {
    std::io::stdout().write_all(bytes)?;
    Ok(())
}

fn write_to_file(filename: String, bytes: &[u8]) -> Result<()> {
    if filename.is_empty() {
        return Err(anyhow!("Filename cannot be empty"));
    }

    let path = PathBuf::from(&filename);
    let file_res = OpenOptions::new()
        .write(true)
        .create_new(true) // Fail if file exists
        .open(&path);

    match file_res {
        Ok(mut f) => f.write_all(bytes)?,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return write_to_timestamped_file(filename, bytes);
        }
        Err(e) => return Err(e)?,
    };

    let success_message = format!("Secret saved to: {}", filename.cyan());
    println!("{success_message}");

    Ok(())
}

fn write_to_timestamped_file(filename: String, bytes: &[u8]) -> Result<()> {
    let timestamp = timestamp::now_string()?;
    let filename_with_timestamp = format!("{filename}.{timestamp}");

    let warn_message = format!(
        "File {filename} already exists. To prevent overriding we use {filename_with_timestamp} instead."
    );
    eprintln!("{}", warn_message.yellow());

    write_to_file(filename_with_timestamp, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_client::test_utils::{MockClient, MockFactory};
    use hakanai_lib::models::Payload;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_print_to_stdout_with_text() {
        let text = "Hello, World!";
        let result = print_to_stdout(&text.as_bytes().to_vec());
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_to_stdout_with_binary() {
        let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        let result = print_to_stdout(&binary_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_to_stdout_empty() {
        let result = print_to_stdout(&vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_to_file_text() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "Test file content";

        write_to_file(
            file_path.to_string_lossy().to_string(),
            &content.as_bytes().to_vec(),
        )?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_to_file_binary() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.bin");
        let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];

        write_to_file(
            file_path.to_string_lossy().to_string(),
            &binary_data.clone(),
        )?;

        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content, binary_data);

        Ok(())
    }

    #[test]
    fn test_write_to_file_empty() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.txt");

        write_to_file(file_path.to_string_lossy().to_string(), &vec![])?;

        assert!(file_path.exists());
        let read_content = fs::read(&file_path)?;
        assert!(read_content.is_empty());

        Ok(())
    }

    #[test]
    fn test_write_to_file_with_subdirectory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;

        let file_path = sub_dir.join("nested.txt");
        let content = "Nested file content";

        write_to_file(
            file_path.to_string_lossy().to_string(),
            &content.as_bytes().to_vec(),
        )?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_to_file_prevents_overwriting() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("overwrite.txt");

        // Write initial content
        fs::write(&file_path, "Initial content")?;

        // Write new content - should create a new file with timestamp
        let new_content = "New content";
        write_to_file(
            file_path.to_string_lossy().to_string(),
            &new_content.as_bytes().to_vec(),
        )?;

        // Original file should still have initial content
        let original_content = fs::read_to_string(&file_path)?;
        assert_eq!(original_content, "Initial content");

        // New file with timestamp should exist and contain new content
        let files: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("overwrite.txt.")
            })
            .collect();

        assert_eq!(files.len(), 1, "Should have created one timestamped file");

        let timestamped_file = &files[0];
        let timestamped_content = fs::read_to_string(timestamped_file.path())?;
        assert_eq!(timestamped_content, new_content);

        Ok(())
    }

    #[test]
    fn test_write_to_file_special_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("file with spaces and !@#$.txt");
        let content = "Special filename content";

        write_to_file(
            file_path.to_string_lossy().to_string(),
            &content.as_bytes().to_vec(),
        )?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_to_file_empty_filename() {
        let result = write_to_file("".to_string(), b"content");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Filename cannot be empty")
        );
    }

    #[test]
    fn test_output_secret_to_stdout() -> Result<()> {
        let content = b"secret content";
        let result = output_secret(content, true, None);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_output_secret_to_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("output.txt");
        let content = b"secret content";

        let result = output_secret(
            content,
            false,
            Some(file_path.to_string_lossy().to_string()),
        );
        assert!(result.is_ok());

        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content, content);
        Ok(())
    }

    #[test]
    fn test_output_secret_defaults_to_stdout() -> Result<()> {
        let content = b"secret content";
        let result = output_secret(content, false, None);
        assert!(result.is_ok());
        Ok(())
    }

    // Integration tests with mock client
    #[tokio::test]
    async fn test_get_successful_to_stdout() -> Result<()> {
        let payload = Payload::from_bytes(b"secret text content", None);
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, true, None).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_successful_to_file_with_payload_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let payload = Payload::from_bytes(b"file content", Some("document.txt".to_string()));
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        // Use temp directory path to avoid writing to current directory
        let filename = temp_dir
            .path()
            .join("document.txt")
            .to_string_lossy()
            .to_string();
        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, false, Some(filename)).await;

        assert!(result.is_ok());

        // Check that file was created with payload filename
        let file_path = temp_dir.path().join("document.txt");
        let content = fs::read(&file_path)?;
        assert_eq!(content, b"file content");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_successful_to_file_with_custom_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let payload = Payload::from_bytes(b"binary content", Some("original.bin".to_string()));
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let custom_filename = temp_dir
            .path()
            .join("custom.bin")
            .to_string_lossy()
            .to_string();
        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, false, Some(custom_filename.clone())).await;

        assert!(result.is_ok());

        // Check that file was created with custom filename
        let file_path = temp_dir.path().join("custom.bin");
        let content = fs::read(&file_path)?;
        assert_eq!(content, b"binary content");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_successful_binary_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let binary_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x42, 0x43];
        let payload = Payload::from_bytes(&binary_data, Some("binary.dat".to_string()));
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let filename = temp_dir
            .path()
            .join("output.dat")
            .to_string_lossy()
            .to_string();
        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, false, Some(filename)).await;

        assert!(result.is_ok());

        let file_path = temp_dir.path().join("output.dat");
        let content = fs::read(&file_path)?;
        assert_eq!(content, binary_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_client_error() -> Result<()> {
        let client = MockClient::new().with_receive_failure("Network timeout".to_string());
        let factory = MockFactory::new().with_client(client);

        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, true, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Network timeout"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_empty_payload() -> Result<()> {
        let payload = Payload::from_bytes(b"", None);
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(factory, url, true, None).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_file_overwrite_prevention() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("existing.txt");

        // Create existing file
        fs::write(&file_path, "existing content")?;

        let payload = Payload::from_bytes(b"new content", None);
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let url = url::Url::parse("https://example.com/s/test123#key")?;
        let result = get(
            factory,
            url,
            false,
            Some(file_path.to_string_lossy().to_string()),
        )
        .await;

        assert!(result.is_ok());

        // Original file should be unchanged
        let original_content = fs::read_to_string(&file_path)?;
        assert_eq!(original_content, "existing content");

        // New timestamped file should exist
        let files: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("existing.txt.")
            })
            .collect();

        assert_eq!(files.len(), 1);
        let timestamped_content = fs::read_to_string(files[0].path())?;
        assert_eq!(timestamped_content, "new content");
        Ok(())
    }
}
