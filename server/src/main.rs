mod app_data;
mod data_store;
mod options;
mod otel;
mod web_api;
mod web_static;

use std::io::Result;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use clap::Parser;
use tracing::{info, instrument, warn};

use crate::app_data::AppData;
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
    let res = HttpServer::new(move || {
        let app_data = AppData {
            data_store: Box::new(data_store.clone()),
            tokens: tokens.clone(),
            max_ttl: args.max_ttl,
        };
        App::new()
            .app_data(web::Data::new(app_data))
            .app_data(web::PayloadConfig::new(
                args.upload_size_limit as usize * 1024 * 1024,
            ))
            .wrap(Logger::new(
                "%a %{X-Forwarded-For}i %t \"%r\" %s %b \"%{User-Agent}i\" %Ts",
            ))
            .wrap(RequestTracing::new())
            .wrap(RequestMetrics::default())
            .wrap(cors_config(args.cors_allowed_origins.clone()))
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add((
                        "Strict-Transport-Security",
                        "max-age=31536000; includeSubDomains",
                    )),
            )
            .route("/s/{id}", web::get().to(get_secret_short))
            .configure(web_static::configure)
            .service(web::scope("/api").configure(web_api::configure))
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await;

    if let Some(handler) = otel_handler {
        handler.shutdown()
    }

    res
}

fn cors_config(allowed_origins: Option<Vec<String>>) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec![http::Method::GET, http::Method::POST])
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
        ])
        .supports_credentials();

    if let Some(allowed_origins) = &allowed_origins {
        for origin in allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}

#[instrument(skip_all)]
async fn get_secret_short(
    http_req: actix_web::HttpRequest,
    req: web::Path<String>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    let user_agent = http_req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    info!("Received request for secret: {}", req);

    if !user_agent.starts_with("hakanai-client") {
        return HttpResponse::Ok()
            .content_type("text/html")
            .body(web_static::SECRET_HTML_CONTENT);
    }

    match web_api::get_secret_from_request(req, app_data).await {
        Ok(secret) => HttpResponse::Ok().body(secret),
        Err(e) => e.error_response(),
    }
}
