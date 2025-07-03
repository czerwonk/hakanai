use anyhow::{Result, anyhow};

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn get(link: url::Url) -> Result<()> {
    let data = client::new()
        .receive_secret(link.clone())
        .await
        .map_err(|e| anyhow!(e))?;
    println!("{data}");

    Ok(())
}
