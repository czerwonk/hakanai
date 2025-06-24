use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn send(server: url::Url, ttl: Duration) -> Result<()> {
    let secret =
        std::io::read_to_string(io::stdin()).with_context(|| "failed to read from stdin")?;

    let link = client::new()
        .send_secret(server.clone(), secret, ttl)
        .await
        .with_context(|| format!("failed to send secret to {}", server))?;

    println!(
        "Secret sent successfully!\nYou can access it at: {}",
        link.to_string().cyan()
    );

    Ok(())
}
