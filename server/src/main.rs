// SPDX-License-Identifier: Apache-2.0

mod admin_user;
mod app_data;
mod data_store;
mod filters;
mod metrics;
mod observer;
mod options;
mod otel;
mod redis_client;
mod size_limit;
mod size_limited_json;
mod stats;
mod token;
mod user;
mod web;

use std::io::Result;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use redis::aio::ConnectionManager;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::metrics::{EventMetrics, MetricsCollector};
use crate::options::Args;
use crate::redis_client::RedisClient;
use crate::stats::StatsStore;
use crate::token::{RedisTokenStore, TokenManager, TokenStore};

#[cfg(test)]
mod test_utils;

/// Connection timeout for Redis operations during startup
const REDIS_CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

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

    info!("Hakanai Server (v{})", env!("CARGO_PKG_VERSION"));

    let redis_con = match connect_to_redis(&args.redis_dsn).await {
        Ok(con) => con,
        Err(e) => {
            eprintln!("Failed to connect to Redis: {e}");
            eprintln!("Please ensure Redis is running and accessible",);
            return Err(std::io::Error::other(e));
        }
    };

    let redis_client = RedisClient::new(redis_con.clone(), args.max_ttl);

    let token_store = token::RedisTokenStore::new(redis_con.clone());
    let token_manager = token::TokenManager::new(token_store.clone());
    if args.reset_admin_token
        && let Err(e) = reset_admin_token(&token_manager).await
    {
        eprintln!("Failed to reset admin token: {e}");
        return Err(std::io::Error::other(e));
    }

    if args.reset_user_tokens
        && let Err(e) = reset_user_tokens(&token_manager).await
    {
        eprintln!("Failed to reset user tokens: {e}");
        return Err(std::io::Error::other(e));
    }

    if args.reset_user_tokens || args.reset_admin_token {
        return Ok(()); // do not start server on reset
    }

    if let Err(e) = initialize_tokens(&token_manager, &args).await {
        eprintln!("Failed to initialize tokens: {e}");
        return Err(std::io::Error::other(e));
    }

    let stats_enabled = args.stats_enabled;
    let stats_ttl = args.stats_ttl;
    let mut options = web::WebServerOptions::new(args);

    if otel_handler.is_some() {
        initialize_metrics(&redis_client, &token_store);
        options = options.with_event_metrics(EventMetrics::new());
    }

    if stats_enabled {
        let stats_store = StatsStore::new(redis_con.clone(), stats_ttl);
        options = options.with_stats_store(stats_store);
    }

    let res = web::run_server(redis_client, token_manager, options).await;

    if let Some(handler) = otel_handler {
        handler.shutdown()
    }

    res
}

async fn connect_to_redis(dsn: &str) -> anyhow::Result<ConnectionManager> {
    info!("Connecting to Redis");

    let client = redis::Client::open(dsn)?;
    let con = timeout(
        REDIS_CONNECTION_TIMEOUT,
        redis::aio::ConnectionManager::new(client),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timed out connecting to Redis"))??;

    Ok(con)
}

async fn reset_user_tokens<T: TokenStore>(token_manager: &TokenManager<T>) -> anyhow::Result<()> {
    let default_token = token_manager.reset_user_tokens().await?;
    info!("Default user token: {default_token}");
    Ok(())
}

async fn reset_admin_token<T: TokenStore>(token_manager: &TokenManager<T>) -> anyhow::Result<()> {
    let admin_token = token_manager.create_admin_token().await?;
    info!("Admin token: {admin_token}");
    Ok(())
}

async fn initialize_tokens<T: TokenStore>(
    token_manager: &TokenManager<T>,
    args: &Args,
) -> anyhow::Result<()> {
    if args.enable_admin_token {
        initialize_admin_token(token_manager).await?;
    }

    initialize_user_tokens(token_manager).await
}

async fn initialize_user_tokens<T: TokenStore>(
    token_manager: &TokenManager<T>,
) -> anyhow::Result<()> {
    if let Some(default_token) = token_manager.create_default_token_if_none().await? {
        info!("Default user token: {default_token}");
    }

    Ok(())
}

async fn initialize_admin_token<T: TokenStore>(
    token_manager: &TokenManager<T>,
) -> anyhow::Result<()> {
    if let Some(admin_token) = token_manager.create_admin_token_if_none().await? {
        info!("Admin token: {admin_token}");
    };

    Ok(())
}

fn initialize_metrics(redis_client: &RedisClient, redis_token_store: &RedisTokenStore) {
    info!("Initializing metrics collection with 30s interval");
    let token_store = Arc::new(redis_token_store.clone());
    let data_store = Arc::new(redis_client.clone());
    let collection_interval = Duration::from_secs(30); // Collect metrics every 30 seconds

    let collector = MetricsCollector::new();
    collector.start_collection(token_store, data_store, collection_interval);

    debug!(
        "Started metrics collection with interval: {:?}",
        collection_interval
    );
}
