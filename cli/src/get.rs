// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;
use std::fs::OpenOptions;
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use colored::Colorize;
use zeroize::Zeroizing;
use zip::ZipArchive;

use hakanai_lib::client::Client;
use hakanai_lib::models::Payload;
use hakanai_lib::options::SecretReceiveOptions;
use hakanai_lib::utils::timestamp;

use crate::args::GetArgs;
use crate::factory::Factory;
use crate::helper;

pub async fn get<T: Factory>(factory: T, args: GetArgs) -> Result<()> {
    args.validate()?;

    let user_agent = helper::get_user_agent_name();
    let observer = factory.new_observer("Receiving secret...")?;
    let mut opts = SecretReceiveOptions::default()
        .with_user_agent(user_agent)
        .with_observer(observer);

    if let Some(ref passphrase) = args.passphrase {
        let bytes = Zeroizing::new(passphrase.bytes().collect::<Vec<u8>>());
        opts = opts.with_passphrase(bytes.as_ref());
    }

    let url = args.secret_url()?.clone();
    let payload = factory.new_client().receive_secret(url, Some(opts)).await?;

    output_secret(payload, args.clone())?;

    Ok(())
}

fn output_secret(payload: Payload, args: GetArgs) -> Result<()> {
    let bytes = Zeroizing::new(payload.decode_bytes()?);
    let filename = args.filename.or_else(|| payload.filename.clone());
    let output_directory = match args.output_dir {
        Some(dir) => dir,
        None => current_dir()?,
    };

    if args.to_stdout {
        print_to_stdout(&bytes)?;
    } else if let Some(name) = payload.filename.clone()
        && args.extract
        && is_archive(&name)
    {
        extract_archive(name, &bytes, &output_directory)?;
    } else if let Some(file) = filename {
        write_to_file(
            file,
            Cursor::<&[u8]>::new(bytes.as_ref()),
            &output_directory,
        )?;
    } else {
        print_to_stdout(&bytes)?;
    }

    Ok(())
}

fn is_archive(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".zip")
}

fn print_to_stdout(bytes: &[u8]) -> Result<()> {
    std::io::stdout().write_all(bytes)?;
    Ok(())
}

fn extract_archive(filename: String, bytes: &[u8], target_dir: &Path) -> Result<()> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    println!("Extracting archive: {}", filename.cyan());
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.is_dir() {
            continue; // skip directories
        }

        let name = file.name().to_string();

        // extract flat, just use the filename
        let flat_name = PathBuf::from(&name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&name)
            .to_string();
        write_to_file(flat_name, file, target_dir)?;
    }

    Ok(())
}

fn write_to_file<T: Read>(filename: String, mut r: T, target_dir: &Path) -> Result<()> {
    if filename.is_empty() {
        return Err(anyhow!("Filename cannot be empty"));
    }

    let path = PathBuf::from(&target_dir).join(filename.clone());
    let file_res = OpenOptions::new()
        .write(true)
        .create_new(true) // fail if file exists
        .open(&path);

    match file_res {
        Ok(mut f) => io::copy(&mut r, &mut f)?,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return write_to_timestamped_file(filename, r, target_dir);
        }
        Err(e) => return Err(e)?,
    };

    let success_message = format!("Saved to: {}", filename.cyan());
    println!("{success_message}");

    Ok(())
}

fn write_to_timestamped_file<T: Read>(filename: String, r: T, target_dir: &Path) -> Result<()> {
    let timestamp = timestamp::now_string()?;
    let filename_with_timestamp = format!("{filename}.{timestamp}");

    let warn_message = format!(
        "File {filename} already exists. To prevent overriding we use {filename_with_timestamp} instead."
    );
    eprintln!("{}", warn_message.yellow());

    write_to_file(filename_with_timestamp, r, target_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use anyhow::Result;
    use tempfile::TempDir;

    use hakanai_lib::client_mock::MockClient;
    use hakanai_lib::models::Payload;

    use crate::factory_mock::test_utils::MockFactory;

    #[tokio::test]
    async fn test_get_successful_to_stdout() -> Result<()> {
        let payload = Payload::from_bytes(b"secret text content");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key").with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_successful_to_file_with_payload_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let payload = Payload::from_bytes(b"file content").with_filename("document.txt");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        // Use temp directory path to avoid writing to current directory
        let filename = temp_dir
            .path()
            .join("document.txt")
            .to_string_lossy()
            .to_string();
        let args = GetArgs::builder("https://example.com/s/test123#key").with_filename(&filename);
        get(factory, args).await?;

        // Check that file was created with payload filename
        let file_path = temp_dir.path().join("document.txt");
        let content = fs::read(&file_path)?;
        assert_eq!(content, b"file content");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_successful_to_file_with_custom_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let payload = Payload::from_bytes(b"binary content").with_filename("original.bin");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let custom_filename = temp_dir
            .path()
            .join("custom.bin")
            .to_string_lossy()
            .to_string();
        let args =
            GetArgs::builder("https://example.com/s/test123#key").with_filename(&custom_filename);
        get(factory, args).await?;

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
        let payload = Payload::from_bytes(&binary_data).with_filename("binary.dat");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let filename = temp_dir
            .path()
            .join("output.dat")
            .to_string_lossy()
            .to_string();
        let args = GetArgs::builder("https://example.com/s/test123#key").with_filename(&filename);
        get(factory, args).await?;

        let file_path = temp_dir.path().join("output.dat");
        let content = fs::read(&file_path)?;
        assert_eq!(content, binary_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_client_error() -> Result<()> {
        let client = MockClient::new().with_receive_failure("Network timeout".to_string());
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key").with_to_stdout();
        let result = get(factory, args).await;

        assert!(result.is_err(), "Expected network error, got: {:?}", result);
        assert!(result.unwrap_err().to_string().contains("Network timeout"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_empty_payload() -> Result<()> {
        let payload = Payload::from_bytes(b"");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key").with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_file_overwrite_prevention() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("existing.txt");

        // Create existing file
        fs::write(&file_path, "existing content")?;

        let payload = Payload::from_bytes(b"new content");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_filename(file_path.to_string_lossy().as_ref());
        get(factory, args).await?;

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

    // Tests for archive extraction
    #[test]
    fn test_is_archive() {
        assert!(is_archive("test.zip"));
        assert!(is_archive("archive.ZIP"));
        assert!(is_archive("my-files.zip"));
        assert!(!is_archive("test.tar"));
        assert!(!is_archive("test.gz"));
        assert!(!is_archive("test.txt"));
        assert!(!is_archive("test"));
    }

    #[test]
    fn test_extract_archive_with_multiple_files() -> Result<()> {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::FileOptions;

        let temp_dir = TempDir::new()?;

        // Create a ZIP archive in memory
        let mut zip_data = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut zip_data));

            // Add files to the archive
            let options = FileOptions::<()>::default();
            zip.start_file("file1.txt", options)?;
            zip.write_all(b"Content of file 1")?;

            zip.start_file("file2.txt", options)?;
            zip.write_all(b"Content of file 2")?;

            zip.add_directory("subdir/", options)?;

            zip.start_file("subdir/file3.txt", options)?;
            zip.write_all(b"Content of file 3 in subdir")?;

            zip.finish()?;
        }

        // Extract to the temp directory
        extract_archive("test.zip".to_string(), &zip_data, temp_dir.path())?;

        // Verify extracted files - all files are extracted flat (no subdirectories)
        assert!(temp_dir.path().join("file1.txt").exists());
        assert!(temp_dir.path().join("file2.txt").exists());
        // subdir/file3.txt is extracted as just file3.txt
        assert!(temp_dir.path().join("file3.txt").exists());

        let content1 = fs::read_to_string(temp_dir.path().join("file1.txt"))?;
        assert_eq!(content1, "Content of file 1");

        let content2 = fs::read_to_string(temp_dir.path().join("file2.txt"))?;
        assert_eq!(content2, "Content of file 2");

        let content3 = fs::read_to_string(temp_dir.path().join("file3.txt"))?;
        assert_eq!(content3, "Content of file 3 in subdir");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_only_for_zip_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test that non-zip files are saved normally even with --extract
        let payload = Payload::from_bytes(b"Not a zip file").with_filename("document.pdf");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_extract()
            .with_output_dir(temp_dir.path().to_string_lossy().as_ref());
        get(factory, args).await?;

        // Should save as regular file, not attempt extraction
        assert!(temp_dir.path().join("document.pdf").exists());
        let content = fs::read(temp_dir.path().join("document.pdf"))?;
        assert_eq!(content, b"Not a zip file");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_with_existing_files() -> Result<()> {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::FileOptions;

        let temp_dir = TempDir::new()?;

        // Create existing file
        fs::write(temp_dir.path().join("file1.txt"), "existing content")?;

        // Create ZIP archive
        let mut zip_data = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut zip_data));

            let options = FileOptions::<()>::default();
            zip.start_file("file1.txt", options)?;
            zip.write_all(b"New content from ZIP")?;

            zip.start_file("file2.txt", options)?;
            zip.write_all(b"Another file")?;

            zip.finish()?;
        }

        let payload = Payload::from_bytes(&zip_data).with_filename("archive.zip");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_extract()
            .with_output_dir(temp_dir.path().to_string_lossy().as_ref());
        get(factory, args).await?;

        // Original file should be unchanged
        let content1 = fs::read_to_string(temp_dir.path().join("file1.txt"))?;
        assert_eq!(content1, "existing content");

        // Should have created timestamped version
        let files: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("file1.txt.")
            })
            .collect();

        assert_eq!(files.len(), 1);
        let timestamped_content = fs::read_to_string(files[0].path())?;
        assert_eq!(timestamped_content, "New content from ZIP");

        // New file should be created normally
        let content2 = fs::read_to_string(temp_dir.path().join("file2.txt"))?;
        assert_eq!(content2, "Another file");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_empty_archive() -> Result<()> {
        use zip::ZipWriter;

        let temp_dir = TempDir::new()?;

        // Create empty ZIP archive
        let mut zip_data = Vec::new();
        {
            let zip = ZipWriter::new(std::io::Cursor::new(&mut zip_data));
            zip.finish()?;
        }

        let payload = Payload::from_bytes(&zip_data).with_filename("empty.zip");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_extract()
            .with_output_dir(temp_dir.path().to_string_lossy().as_ref());
        get(factory, args).await?;

        // No files should be created (empty archive extracts nothing)
        let entries: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(
            entries.len(),
            0,
            "Expected no files after extracting empty archive"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_passphrase() -> Result<()> {
        let payload = Payload::from_bytes(b"protected secret");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase("mypassword")
            .with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_empty_passphrase() -> Result<()> {
        let payload = Payload::from_bytes(b"protected secret");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase("")
            .with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_unicode_passphrase() -> Result<()> {
        let payload = Payload::from_bytes(b"unicode protected secret");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase("パスワード123🔒")
            .with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_without_passphrase() -> Result<()> {
        let payload = Payload::from_bytes(b"unprotected secret");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let args = GetArgs::builder("https://example.com/s/test123#key").with_to_stdout();
        // Should work fine without passphrase when secret doesn't require one
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_passphrase_with_file_output() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let payload = Payload::from_bytes(b"protected file content").with_filename("protected.txt");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let filename = temp_dir
            .path()
            .join("output.txt")
            .to_string_lossy()
            .to_string();
        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase("filepassword")
            .with_filename(&filename);
        get(factory, args).await?;

        // Verify file was created with correct content
        let content = fs::read(temp_dir.path().join("output.txt"))?;
        assert_eq!(content, b"protected file content");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_long_passphrase() -> Result<()> {
        let payload = Payload::from_bytes(b"secret with very long passphrase");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        // Test with a very long passphrase (512 characters)
        let long_passphrase = "a".repeat(512);
        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase(&long_passphrase)
            .with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_passphrase_with_special_characters() -> Result<()> {
        let payload = Payload::from_bytes(b"secret with special chars in passphrase");
        let client = MockClient::new().with_receive_success(payload);
        let factory = MockFactory::new().with_client(client);

        let special_passphrase = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let args = GetArgs::builder("https://example.com/s/test123#key")
            .with_passphrase(special_passphrase)
            .with_to_stdout();
        get(factory, args).await?;
        Ok(())
    }
}
