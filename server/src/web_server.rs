use std::collections::HashMap;
use std::io::Result;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use tracing::{info, instrument};

use crate::app_data::AppData;
use crate::data_store::DataStore;
use crate::hash::hash_string;
use crate::options::Args;
use crate::web_api;
use crate::web_static;

pub async fn run<T>(data_store: T, tokens: Vec<String>, args: Args) -> Result<()>
where
    T: DataStore + Clone + 'static,
{
    info!("Starting server on {}:{}", args.listen_address, args.port);

    HttpServer::new(move || {
        let tokens_map: HashMap<String, ()> = tokens
            .clone()
            .into_iter()
            .map(|t| (hash_string(&t), ()))
            .collect();
        let app_data = AppData {
            data_store: Box::new(data_store.clone()),
            tokens: tokens_map,
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
            .wrap(default_headers())
            .wrap(cors_config(args.cors_allowed_origins.clone()))
            .route("/s/{id}", web::get().to(get_secret_short))
            .route("/ready", web::get().to(ready))
            .configure(web_static::configure)
            .service(web::scope("/api/v1").configure(web_api::configure))
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

fn default_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("X-Frame-Options", "DENY"))
        .add(("X-Content-Type-Options", "nosniff"))
        .add((
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains",
        ))
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

    if !user_agent.starts_with("hakanai-") {
        return web_static::serve_get_secret_html().await;
    }

    match web_api::get_secret_from_request(req, app_data).await {
        Ok(secret) => HttpResponse::Ok().body(secret),
        Err(e) => e.error_response(),
    }
}

async fn ready() -> impl Responder {
    HttpResponse::Ok().body("Ready")
}
