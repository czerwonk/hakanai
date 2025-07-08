use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use colored::Colorize;
use zeroize::Zeroizing;

use hakanai_lib::client;
use hakanai_lib::client::Client;
use hakanai_lib::options::SecretReceiveOptions;

use crate::helper::get_user_agent_name;
use crate::observer::ProgressObserver;

pub async fn get(link: url::Url, to_stdout: bool, filename: Option<String>) -> Result<()> {
    let user_agent = get_user_agent_name();
    let observer = ProgressObserver::new("Receiving secret...")?;
    let opts = SecretReceiveOptions::default()
        .with_user_agent(user_agent)
        .with_observer(Arc::new(observer));

    let payload = client::new()
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

    let mut path = PathBuf::from(filename.clone());
    if path.exists() {
        if path.is_dir() {
            return Err(anyhow!("Cannot write to a directory: {}", path.display()));
        }

        if path.is_file() {
            let timestamped_filename = format!("{}.{}", filename, timestamp()?);
            path = PathBuf::from(timestamped_filename);
        }

        let warn_message = format!(
            "File {} already exists. To prevent overriding we use {} instead.",
            filename,
            path.display()
        );
        eprintln!("{}", warn_message.yellow());
    }

    OpenOptions::new()
        .write(true)
        .create_new(true) // Fail if file exists
        .open(&path)?
        .write_all(bytes)?;

    Ok(())
}

fn timestamp() -> Result<String> {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    Ok(format!("{}", duration.as_secs()))
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
