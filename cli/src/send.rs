use std::io;
use std::time::Duration;

use anyhow::{Result, anyhow};
use colored::Colorize;

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn send(server: url::Url, ttl: Duration) -> Result<()> {
    let secret = std::io::read_to_string(io::stdin()).map_err(|e| anyhow!(e))?;

    let link = client::new()
        .send_secret(server.clone(), secret, ttl)
        .await
        .map_err(|e| anyhow!(e))?;

    println!(
        "Secret sent successfully!\nYou can access it at: {}",
        link.to_string().cyan()
    );

    Ok(())
}
