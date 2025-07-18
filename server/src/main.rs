mod app_data;
mod data_store;
mod hash;
mod options;
mod otel;
mod redis_client;
mod token;
mod web_api;
mod web_server;
mod web_static;

use std::io::Result;

use clap::Parser;
use tracing::{info, warn};

use crate::options::Args;
use crate::redis_client::RedisClient;

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();

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
    let default_token_res = token_manager.create_default_token_if_none().await;
    match default_token_res {
        Ok(Some(token)) => info!("New default token created: {token}"),
        Ok(None) => {
            info!("Default token already exists, no token created");
        }
        Err(e) => {
            eprintln!("Failed to access token store: {e}");
            return Err(std::io::Error::other(e));
        }
    }

    let res = web_server::run(redis_client, token_manager, args).await;

    if let Some(handler) = otel_handler {
        handler.shutdown()
    }

    res
}
