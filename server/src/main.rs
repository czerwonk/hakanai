mod app_data;
mod data_store;
mod hash;
mod options;
mod otel;
mod web_api;
mod web_server;
mod web_static;

use std::io::Result;

use clap::Parser;
use tracing::{info, warn};

use crate::data_store::RedisDataStore;
use crate::options::Args;

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
    let data_store = match RedisDataStore::new(&args.redis_dsn, args.max_ttl).await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to create Redis data store: {e}");
            return Err(std::io::Error::other(e));
        }
    };

    let tokens = args.tokens.clone().unwrap_or_default();
    if tokens.is_empty() {
        warn!("No tokens provided, anyone can create secrets.");
    }

    let res = web_server::run(data_store, tokens, args).await;

    if let Some(handler) = otel_handler {
        handler.shutdown()
    }

    res
}
