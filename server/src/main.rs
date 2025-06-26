mod api;
mod app_data;
mod data_store;
mod options;

use std::io::Result;

use actix_web::middleware::{Compat, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use clap::Parser;
use tracing::{info, warn};
use tracing_actix_web::TracingLogger;

use crate::app_data::AppData;
use crate::data_store::RedisDataStore;
use crate::options::Args;

const SECRET_HTML_CONTENT: &str = include_str!("includes/get-secret.html");

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
        let app_data = AppData {
            data_store: Box::new(data_store.clone()),
            tokens: tokens.clone(),
        };
        App::new()
            .app_data(web::Data::new(app_data))
            .wrap(Logger::new(
                "%a %{X-Forwarded-For}i %t \"%r\" %s %b \"%{User-Agent}i\" %Ts",
            ))
            .wrap(Compat::new(TracingLogger::default()))
            .route("/", web::get().to(serve_get_secret_html))
            .route("/s/{id}", web::get().to(get_secret_short))
            .route("/create", web::get().to(serve_create_secret_html))
            .route("/scripts/hakanai-client.js", web::get().to(serve_js_client))
            .route("/style.css", web::get().to(serve_css))
            .route("/icon.svg", web::get().to(serve_icon))
            .route("/logo.svg", web::get().to(serve_logo))
            .service(web::scope("/api").configure(|c| {
                api::configure(c);
            }))
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

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
            .body(SECRET_HTML_CONTENT);
    }

    match api::get_secret_from_request(req, app_data).await {
        Ok(secret) => HttpResponse::Ok().body(secret),
        Err(e) => e.error_response(),
    }
}

async fn serve_js_client() -> impl Responder {
    const CONTENT: &str = include_str!("includes/hakanai-client.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(CONTENT)
}

async fn serve_css() -> impl Responder {
    const CONTENT: &str = include_str!("includes/style.css");
    HttpResponse::Ok().content_type("text/css").body(CONTENT)
}

async fn serve_logo() -> impl Responder {
    const CONTENT: &str = include_str!("../../logo.svg");
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(CONTENT)
}

async fn serve_icon() -> impl Responder {
    const CONTENT: &str = include_str!("../../icon.svg");
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(CONTENT)
}

async fn serve_get_secret_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(SECRET_HTML_CONTENT)
}

async fn serve_create_secret_html() -> impl Responder {
    const CONTENT: &str = include_str!("includes/create-secret.html");
    HttpResponse::Ok().content_type("text/html").body(CONTENT)
}
