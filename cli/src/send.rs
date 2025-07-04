use std::io::{self, Read};
use std::time::Duration;

use anyhow::{Result, anyhow};
use colored::Colorize;

use hakanai_lib::client;
use hakanai_lib::client::Client;
use hakanai_lib::models::Payload;

pub async fn send(
    server: url::Url,
    ttl: Duration,
    token: String,
    file: Option<String>,
) -> Result<()> {
    let secret = read_secret(file)?;
    if secret.is_empty() {
        return Err(anyhow!(
            "No secret provided. Please input a secret to send."
        ));
    }

    if ttl.as_secs() == 0 {
        return Err(anyhow!("TTL must be greater than zero seconds."));
    }

    if token.is_empty() {
        eprintln!("{}", "Warning: No token provided.".yellow());
    }

    let payload = Payload::from_data(secret);
    let link = client::new()
        .send_secret(server.clone(), payload, ttl, token)
        .await
        .map_err(|e| anyhow!(e))?;

    println!(
        "Secret sent successfully!\nYou can access it at: {}",
        link.to_string().cyan()
    );

    Ok(())
}

fn read_secret(file: Option<String>) -> Result<String> {
    if let Some(file_path) = file {
        Ok(std::fs::read_to_string(file_path)?)
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }
}
