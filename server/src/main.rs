mod api;
mod data_store;
mod options;

use std::io::Result;

use actix_web::middleware::{Compat, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use clap::Parser;
use tracing::{info, warn};
use tracing_actix_web::TracingLogger;

use crate::data_store::RedisDataStore;
use crate::options::Args;

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!("Connecting to Redis at {}", args.redis_dsn);
    let data_store: RedisDataStore = match RedisDataStore::new(&args.redis_dsn).await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to create Redis data store: {}", e);
            return Err(std::io::Error::other(e));
        }
    };

    let tokens = args.tokens.unwrap_or_default();
    if tokens.is_empty() {
        warn!("No tokens provided, anyone can create secrets.");
    }

    info!("Starting server on {}:{}", args.listen_address, args.port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compat::new(TracingLogger::default()))
            .route("/", web::get().to(serve_get_secret_html))
            .route("/healthz", web::get().to(healthz))
            .route("/scripts/hakanai-client.js", web::get().to(serve_js_client))
            .service(
                web::scope("/api").configure(|cfg| {
                    api::configure(cfg, Box::new(data_store.clone()), tokens.clone())
                }),
            )
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("healthy")
}

async fn serve_js_client() -> impl Responder {
    const JS_CLIENT: &str = include_str!("includes/hakanai-client.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(JS_CLIENT)
}

async fn serve_get_secret_html() -> impl Responder {
    const HTML_CONTENT: &str = include_str!("includes/get-secret.html");
    HttpResponse::Ok().body(HTML_CONTENT)
}
