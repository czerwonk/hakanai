// SPDX-License-Identifier: Apache-2.0

use core::option::Option;
use std::io::Result;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use tracing::{error, info, instrument};

use super::admin_api;
use super::app_data::{AnonymousOptions, AppData};
use super::size_limit;
use super::web_api;
use super::web_assets::AssetManager;
use super::web_routes;
use crate::metrics::{EventMetrics, MetricsObserver};
use crate::observer::{ObserverManager, WebhookObserver};
use crate::options::{Args, WebhookArgs};
use crate::secret::SecretStore;
use crate::stats::{RedisStatsStore, StatsObserver};
use crate::token::{TokenCreator, TokenValidator};

pub struct WebServerOptions {
    args: Args,
    event_metrics: Option<EventMetrics>,
    stats_store: RedisStatsStore,
}

impl WebServerOptions {
    pub fn new(args: Args, stats_store: RedisStatsStore) -> Self {
        Self {
            args,
            stats_store,
            event_metrics: None,
        }
    }

    pub fn with_event_metrics(mut self, metrics: EventMetrics) -> Self {
        self.event_metrics = Some(metrics);
        self
    }
}

/// Starts the web server with the provided data store and tokens.
pub async fn run_server<D, T>(
    secret_store: D,
    token_manager: T,
    options: WebServerOptions,
) -> Result<()>
where
    D: SecretStore + Clone + 'static,
    T: TokenValidator + TokenCreator + Clone + 'static,
{
    let args = options.args;
    info!("Starting server on {}:{}", args.listen_address, args.port);

    let anonymous_usage = AnonymousOptions {
        allowed: args.allow_anonymous,
        upload_size_limit: args.anonymous_upload_size_limit,
    };

    let impressum_html = build_impressum_html(&args)?;
    let privacy_html = build_privacy_html(&args)?;

    let webhook_args_opt = args.webhook_args().clone();

    HttpServer::new(move || {
        let mut observer_manager = ObserverManager::new();
        if let Some(ref webhook_args) = webhook_args_opt {
            add_webhook_observer(&mut observer_manager, webhook_args);
        }
        if let Some(event_metrics) = &options.event_metrics {
            let metrics_observer = MetricsObserver::new(event_metrics.clone());
            observer_manager.register_observer(Box::new(metrics_observer));
        }

        let mut stats_observer = StatsObserver::new(options.stats_store.clone());
        if let Some(event_metrics) = options.event_metrics.clone() {
            stats_observer = stats_observer.with_event_metrics(event_metrics);
        }
        observer_manager.register_observer(Box::new(stats_observer));

        let asset_manager = AssetManager::new(args.custom_assets_dir.clone());
        let app_data = AppData {
            secret_store: Box::new(secret_store.clone()),
            token_validator: Box::new(token_manager.clone()),
            token_creator: Box::new(token_manager.clone()),
            max_ttl: args.max_ttl,
            anonymous_usage: anonymous_usage.clone(),
            impressum_html: impressum_html.clone(),
            privacy_html: privacy_html.clone(),
            observer_manager,
            show_token_input: args.show_token_input,
            trusted_ip_ranges: args.trusted_ip_ranges.clone(),
            trusted_ip_header: args.trusted_ip_header.clone(),
            country_header: args.country_header.clone(),
            asn_header: args.asn_header.clone(),
            upload_size_limit: args.upload_size_limit,
        };
        let size_limit = size_limit::calculate(args.upload_size_limit);
        App::new()
            .app_data(web::Data::new(app_data))
            .app_data(web::PayloadConfig::new(size_limit))
            .app_data(web::JsonConfig::default().limit(size_limit))
            .app_data(web::Data::new(asset_manager))
            .wrap(Logger::new("%a %{X-Forwarded-For}i %t \"%r\" %s %b %Ts"))
            .wrap(RequestTracing::new())
            .wrap(RequestMetrics::default())
            .wrap(default_headers())
            .wrap(cors_config(args.cors_allowed_origins.clone()))
            .route("/s/{id}", web::get().to(get_secret_short))
            .route("/healthy", web::get().to(healthy))
            .route("/ready", web::get().to(ready))
            .configure(web_routes::configure)
            .service(
                web::scope("/api/v1")
                    .wrap(DefaultHeaders::new().add((
                        "Cache-Control",
                        "no-cache, no-store, must-revalidate, no-transform",
                    )))
                    .configure(|cfg| {
                        web_api::configure(cfg);
                        if args.enable_admin_token {
                            admin_api::configure_routes(cfg);
                        }
                    }),
            )
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

fn add_webhook_observer(observer_manager: &mut ObserverManager, webhook_args: &WebhookArgs) {
    let res = WebhookObserver::new(
        webhook_args.url.clone(),
        webhook_args.token.clone(),
        webhook_args.headers.clone(),
    );
    match res {
        Ok(observer) => {
            observer_manager.register_observer(Box::new(observer));
        }
        Err(e) => {
            error!("Failed to initialize webhook observer: {e}");
        }
    }
}

fn build_impressum_html(args: &Args) -> Result<Option<String>> {
    Ok(match args.load_impressum_content()? {
        Some(content) => {
            info!(
                "Building impressum HTML ({} bytes of content)",
                content.len()
            );
            let template = include_str!("../../includes/impressum.html");
            Some(template.replace(
                r#"<div id="impressum-content-placeholder"></div>"#,
                &content,
            ))
        }
        None => None,
    })
}

fn build_privacy_html(args: &Args) -> Result<Option<String>> {
    Ok(match args.load_privacy_content()? {
        Some(content) => {
            info!(
                "Building privacy policy HTML ({} bytes of content)",
                content.len()
            );
            let template = include_str!("../../includes/privacy.html");
            Some(template.replace(r#"<div id="privacy-content-placeholder"></div>"#, &content))
        }
        None => None,
    })
}

fn default_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("X-Frame-Options", "DENY"))
        .add(("X-Content-Type-Options", "nosniff"))
        .add((
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains",
        ))
        .add(("Content-Security-Policy", "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data: blob:; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests"))
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
        return web_routes::serve_get_secret_html().await;
    }

    match web_api::get_secret_from_request(http_req, req, app_data).await {
        Ok(secret) => HttpResponse::Ok().body(secret),
        Err(e) => e.error_response(),
    }
}

async fn healthy(app_data: web::Data<AppData>) -> impl Responder {
    let res = app_data.secret_store.is_healthy().await;

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
