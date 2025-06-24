use anyhow::{Context, Result};

use hakanai_lib::client;
use hakanai_lib::client::Client;

pub async fn get(link: url::Url) -> Result<()> {
    let data = client::new()
        .receive_secret(link.clone())
        .await
        .with_context(|| format!("failed to get secret from {}", link))?;
    println!("{}", data);

    Ok(())
}
