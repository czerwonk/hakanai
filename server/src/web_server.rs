use std::io::Result;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use tracing::{error, info, instrument};

use crate::admin_api;
use crate::app_data::{AnonymousOptions, AppData};
use crate::data_store::DataStore;
use crate::options::Args;
use crate::token::{TokenCreator, TokenValidator};
use crate::web_api;
use crate::web_static;

/// Starts the web server with the provided data store and tokens.
pub async fn run<D, T>(data_store: D, token_manager: T, args: Args) -> Result<()>
where
    D: DataStore + Clone + 'static,
    T: TokenValidator + TokenCreator + Clone + 'static,
{
    info!("Starting server on {}:{}", args.listen_address, args.port);

    let anonymous_usage = AnonymousOptions {
        allowed: args.allow_anonymous,
        upload_size_limit: args.anonymous_upload_size_limit * 1024,
    };

    HttpServer::new(move || {
        let app_data = AppData {
            data_store: Box::new(data_store.clone()),
            token_validator: Box::new(token_manager.clone()),
            token_creator: Box::new(token_manager.clone()),
            max_ttl: args.max_ttl,
            anonymous_usage: anonymous_usage.clone(),
        };
        App::new()
            .app_data(web::Data::new(app_data))
            .app_data(web::PayloadConfig::new(
                args.upload_size_limit as usize * 1024,
            ))
            .app_data(
                web::JsonConfig::default().limit(args.upload_size_limit as usize * 1024 * 1024),
            )
            .wrap(Logger::new("%a %{X-Forwarded-For}i %t \"%r\" %s %b %Ts"))
            .wrap(RequestTracing::new())
            .wrap(RequestMetrics::default())
            .wrap(default_headers())
            .wrap(cors_config(args.cors_allowed_origins.clone()))
            .route("/s/{id}", web::get().to(get_secret_short))
            .route("/healthy", web::get().to(healthy))
            .route("/ready", web::get().to(ready))
            .configure(web_static::configure)
            .service(web::scope("/api/v1").configure(|cfg| {
                web_api::configure(cfg);
                if args.enable_admin_token {
                    admin_api::configure_routes(cfg);
                }
            }))
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
        .add(("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests"))
        .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
        .add((
            "Permissions-Policy",
            "geolocation=(), microphone=(), camera=()",
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

async fn healthy(app_data: web::Data<AppData>) -> impl Responder {
    let res = app_data.data_store.is_healthy().await;

    match res {
        Ok(()) => HttpResponse::Ok().body("healthy"),
        Err(e) => {
            error!("Health check failed: {e}");
            HttpResponse::InternalServerError().body("unhealthy")
        }
    }
}

async fn ready() -> impl Responder {
    HttpResponse::Ok().body("ready")
}
