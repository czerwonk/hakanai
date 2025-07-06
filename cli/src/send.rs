use std::io::{self, Read};
use std::time::Duration;

use anyhow::{Result, anyhow};
use colored::Colorize;

use hakanai_lib::client;
use hakanai_lib::client::Client;
use hakanai_lib::models::Payload;

const MAX_SECRET_SIZE_MB: usize = 10; // 10 MB

pub async fn send(
    server: url::Url,
    ttl: Duration,
    token: String,
    file: Option<String>,
    as_file: bool,
    file_name: Option<String>,
) -> Result<()> {
    let bytes = read_secret(file.clone())?;

    if bytes.is_empty() {
        return Err(anyhow!(
            "No secret provided. Please input a secret to send."
        ));
    }

    if bytes.len() > MAX_SECRET_SIZE_MB * 1024 * 1024 {
        return Err(anyhow!(
            "Secret size exceeds the maximum limit of {MAX_SECRET_SIZE_MB} megabytes."
        ));
    }

    if ttl.as_secs() == 0 {
        return Err(anyhow!("TTL must be greater than zero seconds."));
    }

    if token.is_empty() {
        eprintln!("{}", "Warning: No token provided.".yellow());
    }

    let file_name = get_file_name(file, as_file, file_name)?;
    let payload = Payload::from_bytes(&bytes, file_name);

    let link = client::new()
        .send_secret(server.clone(), payload, ttl, token, None)
        .await
        .map_err(|e| anyhow!(e))?;

    println!(
        "Secret sent successfully!\nYou can access it at: {}",
        link.to_string().cyan()
    );

    Ok(())
}

fn get_file_name(
    file: Option<String>,
    as_file: bool,
    file_name: Option<String>,
) -> Result<Option<String>> {
    if !as_file {
        return Ok(None);
    }

    if let Some(name) = file_name {
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
