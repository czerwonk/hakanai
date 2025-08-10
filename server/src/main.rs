mod admin_api;
mod app_data;
mod data_store;
mod metrics;
mod observer;
mod options;
mod otel;
mod redis_client;
#[cfg(test)]
mod test_utils;
mod token;
mod web_api;
mod web_server;
mod web_static;
mod webhook_observer;

use std::io::Result;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tracing::{info, warn};

use crate::options::Args;
use crate::redis_client::RedisClient;
use crate::token::TokenManager;

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Invalid config: {e}");
        return Err(std::io::Error::other(e));
    }

    let otel_handler = match otel::init() {
        Ok(handler) => handler,
        Err(err) => {
            warn!("Failed to initialize OpenTelemetry: {}", err);
            None
        }
    };

    info!("Connecting to Redis");
    let redis_client = match RedisClient::new(&args.redis_dsn, args.max_ttl).await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to create Redis data store: {e}");
            return Err(std::io::Error::other(e));
        }
    };

    let token_manager = token::TokenManager::new(redis_client.clone());

    if let Err(e) = initialize_tokens(&token_manager, &args).await {
        eprintln!("Failed to initialize tokens: {e}");
        return Err(std::io::Error::other(e));
    }

    if otel_handler.is_some() {
        initialize_metrics(&redis_client);
    }

    let res = web_server::run(redis_client, token_manager, args).await;

    if let Some(handler) = otel_handler {
        handler.shutdown()
    }

    res
}

async fn initialize_tokens(
    token_manager: &TokenManager<RedisClient>,
    args: &Args,
) -> anyhow::Result<()> {
    initialize_admin_token(token_manager, args).await?;
    initialize_user_tokens(token_manager, args).await
}

async fn initialize_user_tokens(
    token_manager: &TokenManager<RedisClient>,
    args: &Args,
) -> anyhow::Result<()> {
    if args.reset_user_tokens {
        info!("Resetting user tokens");
        let default_token = token_manager.reset_user_tokens().await?;
        info!("Default user token: {default_token}");
    }

    if let Some(default_token) = token_manager.create_default_token_if_none().await? {
        info!("Default user token: {default_token}");
    }

    Ok(())
}

async fn initialize_admin_token(
    token_manager: &TokenManager<RedisClient>,
    args: &Args,
) -> anyhow::Result<()> {
    if !args.enable_admin_token {
        return Ok(());
    }

    let new_admin_token: Option<String> = if args.reset_admin_token {
        Some(token_manager.create_admin_token().await?)
    } else {
        token_manager.create_admin_token_if_none().await?
    };

    if let Some(admin_token) = new_admin_token {
        info!("Admin token: {admin_token}");
    };

    Ok(())
}

fn initialize_metrics(redis_client: &RedisClient) {
    info!("Initializing metrics collection with 30s interval");
    let redis_token_store = Arc::new(redis_client.clone());
    let redis_data_store = Arc::new(redis_client.clone());
    let collection_interval = Duration::from_secs(30); // Collect metrics every 30 seconds

    crate::metrics::init_metrics_collection(
        redis_token_store,
        redis_data_store,
        collection_interval,
    );
}
