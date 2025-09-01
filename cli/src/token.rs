// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, anyhow};
use colored::Colorize;
use rpassword::prompt_password;

use hakanai_lib::models::{CreateTokenRequest, CreateTokenResponse};

use crate::cli::TokenArgs;
use crate::helper;

pub async fn token(args: TokenArgs) -> Result<()> {
    let admin_token = prompt_password("Enter admin token: ")?;
    if admin_token.is_empty() {
        return Err(anyhow!("Admin token cannot be empty"));
    }

    let resp = create_token_request(&admin_token, &args).await?;

    println!("\n{}", "Token created successfully!".green().bold());
    println!("\n{}", "User token:".bold());
    println!("{}", resp.token.cyan());
    Ok(())
}

async fn create_token_request(admin_token: &str, args: &TokenArgs) -> Result<CreateTokenResponse> {
    let request = CreateTokenRequest {
        upload_size_limit: args.limit,
        ttl_seconds: args.ttl.as_secs(),
    };

    let client = reqwest::Client::new();
    let url = args.server.join("api/v1/admin/tokens")?;

    let response = client
        .post(url)
        .header("User-Agent", helper::get_user_agent_name())
        .header("Authorization", format!("Bearer {admin_token}"))
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!(
            "Failed to create token: {} - {}",
            status,
            error_text
        ));
    }

    Ok(response.json().await?)
}
