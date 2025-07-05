use anyhow::{Result, anyhow};

use hakanai_lib::client;
use hakanai_lib::client::Client;

use std::io::Write;

pub async fn get(link: url::Url) -> Result<()> {
    let payload = client::new()
        .receive_secret(link.clone())
        .await
        .map_err(|e| anyhow!(e))?;

    let bytes = payload.decode_bytes()?;
    std::io::stdout().write_all(&bytes)?;

    Ok(())
}
