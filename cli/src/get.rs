use anyhow::{Result, anyhow};

use hakanai_lib::client;
use hakanai_lib::client::Client;

use std::io::Write;

pub async fn get(link: url::Url, to_stdout: bool, filename: Option<String>) -> Result<()> {
    let payload = client::new()
        .receive_secret(link.clone())
        .await
        .map_err(|e| anyhow!(e))?;

    let bytes = payload.decode_bytes()?;
    if to_stdout {
        print_to_stdout(bytes)?;
    } else if let Some(file) = filename {
        write_to_file(file, bytes)?;
    } else if let Some(file) = payload.filename {
        write_to_file(file, bytes)?;
    } else {
        print_to_stdout(bytes)?;
    }

    Ok(())
}

fn print_to_stdout(bytes: Vec<u8>) -> Result<()> {
    std::io::stdout().write_all(&bytes)?;
    Ok(())
}

fn write_to_file(filename: String, bytes: Vec<u8>) -> Result<()> {
    let mut file = std::fs::File::create(filename)?;
    file.write_all(&bytes)?;
    Ok(())
}
