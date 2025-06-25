use std::io;
use std::time::Duration;

use anyhow::{Result, anyhow};
use colored::Colorize;

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn send(server: url::Url, ttl: Duration, token: String) -> Result<()> {
    let secret = std::io::read_to_string(io::stdin()).map_err(|e| anyhow!(e))?;
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

    let link = client::new()
        .send_secret(server.clone(), secret, ttl, token)
        .await
        .map_err(|e| anyhow!(e))?;

    println!(
        "Secret sent successfully!\nYou can access it at: {}",
        link.to_string().cyan()
    );

    Ok(())
}
