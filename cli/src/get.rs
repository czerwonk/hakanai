use anyhow::{Result, anyhow};

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn get(link: url::Url) -> Result<()> {
    let payload = client::new()
        .receive_secret(link.clone())
        .await
        .map_err(|e| anyhow!(e))?;

    let text = payload
        .decode_text()
        .map_err(|e| anyhow!("Failed to decode text data: {e}"))?;
    print!("{text}");

    Ok(())
}
